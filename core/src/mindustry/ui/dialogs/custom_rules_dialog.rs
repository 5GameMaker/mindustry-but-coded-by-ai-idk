//! Custom-rules dialog model mirroring upstream `mindustry.ui.dialogs.CustomRulesDialog`.

use std::collections::{BTreeMap, BTreeSet};

use crate::mindustry::{
    content::{planets::PlanetContent, weathers::WeatherContent},
    game::{vanilla_teams, Rules, Team, TeamRegistry, TeamRule, TEAM_CRUX, TEAM_MALIS},
    r#type::{ItemStack, Weather, WeatherEntry},
    ui::upstream_menu_bundle_value_for_locale,
    vars::TILE_SIZE,
    world::meta::Env,
};

pub const CUSTOM_RULES_DIALOG_TITLE: &str = "@mode.custom";
pub const CUSTOM_RULES_EDIT_BUTTON_TEXT: &str = "@edit";
pub const CUSTOM_RULES_EDIT_BUTTON_ICON: &str = "pencil";
pub const CUSTOM_RULES_EDIT_DIALOG_TITLE: &str = "@waves.edit";
pub const CUSTOM_RULES_SEARCH_LABEL: &str = "@search";
pub const CUSTOM_RULES_SEARCH_CLEAR_ICON: &str = "cancel";
pub const CUSTOM_RULES_EDIT_TABLE_BACKGROUND: &str = "Tex.button";
pub const CUSTOM_RULES_EDIT_BUTTON_STYLE: &str = "Styles.cleart";
pub const CUSTOM_RULES_EDIT_BUTTON_SIZE: (f32, f32) = (280.0, 64.0);
pub const CUSTOM_RULES_EDIT_BUTTON_MARGIN_LEFT: f32 = 12.0;
pub const CUSTOM_RULES_RULE_BUTTON_WIDTH: f32 = 300.0;
pub const CUSTOM_RULES_SMALL_BUTTON_WIDTH: f32 = 250.0;
pub const CUSTOM_RULES_TEAM_BUTTON_SIZE: f32 = 60.0;
pub const CUSTOM_RULES_TEAM_SECTION_BUTTON_SIZE: (f32, f32) = (260.0, 55.0);
pub const CUSTOM_RULES_PLANET_TABLE_BACKGROUND: &str = "Tex.button";
pub const CUSTOM_RULES_PLANET_BUTTON_STYLE: &str = "Styles.flatTogglet";
pub const CUSTOM_RULES_PLANET_BUTTON_SIZE: (f32, f32) = (140.0, 50.0);
pub const CUSTOM_RULES_PLANET_COLUMNS: usize = 3;
pub const CUSTOM_RULES_WEATHER_CARD_WIDTH: f32 = 410.0;
pub const CUSTOM_RULES_WEATHER_COLUMN_WIDTH: f32 = 450.0;
pub const CUSTOM_RULES_WEATHER_ADD_BUTTON_SIZE: (f32, f32) = (140.0, 50.0);
pub const CUSTOM_RULES_WEATHER_ADD_COLUMNS: usize = 2;
pub const CUSTOM_RULES_LOADOUT_CAPACITY: i32 = 999_999;
pub const CUSTOM_RULES_DEFAULT_LOADOUT_ITEM: &str = "copper";
pub const CUSTOM_RULES_DEFAULT_LOADOUT_AMOUNT: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CustomRulesDialogContext {
    pub mobile: bool,
    pub portrait: bool,
    pub graphics_width: f32,
    pub scl: f32,
    pub state_is_game: bool,
}

impl Default for CustomRulesDialogContext {
    fn default() -> Self {
        Self {
            mobile: false,
            portrait: false,
            graphics_width: 1280.0,
            scl: 1.0,
            state_is_game: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesDialogModel {
    pub title: &'static str,
    pub fill_parent: bool,
    pub close_button_added: bool,
    pub edit_button: CustomRulesEditButton,
    pub search: CustomRulesSearchModel,
    pub scroll_x: bool,
    pub categories: Vec<CustomRulesSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesEditButton {
    pub text: &'static str,
    pub icon: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesSearchModel {
    pub label: &'static str,
    pub text: String,
    pub clear_icon: &'static str,
    pub keyboard_focused: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesSection {
    pub category: CustomRulesCategory,
    pub name: &'static str,
    pub title_key: String,
    pub title: String,
    pub rows: Vec<CustomRulesRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesCategory {
    Waves,
    ResourcesBuilding,
    Unit,
    Enemy,
    Environment,
    Planet,
    Teams,
}

impl CustomRulesCategory {
    pub const JAVA_ORDER: [Self; 7] = [
        Self::Waves,
        Self::ResourcesBuilding,
        Self::Unit,
        Self::Enemy,
        Self::Environment,
        Self::Planet,
        Self::Teams,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Waves => "waves",
            Self::ResourcesBuilding => "resourcesbuilding",
            Self::Unit => "unit",
            Self::Enemy => "enemy",
            Self::Environment => "environment",
            Self::Planet => "planet",
            Self::Teams => "teams",
        }
    }

    pub fn title_key(self) -> String {
        format!("rules.title.{}", self.name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesRow {
    pub label_key: String,
    pub label: String,
    pub enabled: bool,
    pub info: Option<CustomRulesInfo>,
    pub kind: CustomRulesRowKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomRulesRowKind {
    Check {
        kind: CustomRulesCheckKind,
        checked: bool,
    },
    Number {
        kind: CustomRulesNumberKind,
        value: f32,
        value_text: String,
        min: f32,
        max: f32,
    },
    Integer {
        kind: CustomRulesIntegerKind,
        value: i32,
        min: i32,
        max: i32,
    },
    TeamSelect {
        kind: CustomRulesTeamSelectKind,
        selected_team: u8,
        buttons: Vec<CustomRulesTeamButton>,
    },
    Button {
        action: CustomRulesButtonAction,
        width: f32,
    },
    Color {
        kind: CustomRulesColorKind,
        rgba: [f32; 4],
        width: f32,
    },
    PlanetSelector(CustomRulesPlanetSelectorModel),
    TeamRules(CustomRulesTeamSection),
    AdditionalCheck {
        index: usize,
        tag_key: String,
        checked: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomRulesInfo {
    pub key: String,
    pub presentation: CustomRulesInfoPresentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesInfoPresentation {
    Tooltip,
    MobileInfoButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesButtonAction {
    OpenLoadout,
    OpenBannedBlocks,
    OpenBannedUnits,
    OpenWeather,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesColorKind {
    AmbientLight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesPlanetSelectorModel {
    pub background: &'static str,
    pub button_style: &'static str,
    pub button_size: (f32, f32),
    pub columns: usize,
    pub options: Vec<CustomRulesPlanetButton>,
    pub any_env: CustomRulesPlanetButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesPlanetButton {
    pub index: usize,
    pub name: String,
    pub label: String,
    pub env: u32,
    pub checked: bool,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesTeamButton {
    pub id: u8,
    pub name: String,
    pub color_rgba: u32,
    pub colored_name: String,
    pub checked: bool,
    pub size: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesTeamSection {
    pub team: CustomRulesTeamButton,
    pub expanded: bool,
    pub button_size: (f32, f32),
    pub rows: Vec<CustomRulesTeamRuleRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesTeamRuleRow {
    pub label_key: String,
    pub label: String,
    pub enabled: bool,
    pub info: Option<CustomRulesInfo>,
    pub field: CustomRulesTeamRuleField,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomRulesTeamRuleField {
    Check {
        kind: CustomRulesTeamCheckKind,
        checked: bool,
    },
    Number {
        kind: CustomRulesTeamNumberKind,
        value: f32,
        value_text: String,
        min: f32,
        max: f32,
    },
    Integer {
        kind: CustomRulesTeamIntegerKind,
        value: i32,
        min: i32,
        max: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesCheckKind {
    Waves,
    WaveSending,
    WaveTimer,
    WaitEnemies,
    RandomWaveAi,
    WavesSpawnAtCores,
    AirUseSpawns,
    AllowEditWorldProcessors,
    InfiniteResources,
    OnlyDepositCore,
    AllowCoreUnloaders,
    DerelictRepair,
    ReactorExplosions,
    SchematicsAllowed,
    CoreIncinerates,
    CleanupDeadTeams,
    DisableWorldProcessors,
    HideBannedBlocks,
    BlockWhitelist,
    UnitCapVariable,
    UnitPayloadsExplode,
    LogicUnitControl,
    LogicUnitBuild,
    LogicUnitDeconstruct,
    UnitWhitelist,
    AttackMode,
    CoreCapture,
    PlaceRangeCheck,
    PolygonCoreProtection,
    PauseDisabled,
    DamageExplosions,
    Fire,
    Fog,
    Lighting,
    LimitMapArea,
    AllowEditRules,
}

impl CustomRulesCheckKind {
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Waves => "@rules.waves",
            Self::WaveSending => "@rules.wavesending",
            Self::WaveTimer => "@rules.wavetimer",
            Self::WaitEnemies => "@rules.waitForWaveToEnd",
            Self::RandomWaveAi => "@rules.randomwaveai",
            Self::WavesSpawnAtCores => "@rules.wavespawnatcores",
            Self::AirUseSpawns => "@rules.airUseSpawns",
            Self::AllowEditWorldProcessors => "@rules.alloweditworldprocessors",
            Self::InfiniteResources => "@rules.infiniteresources",
            Self::OnlyDepositCore => "@rules.onlydepositcore",
            Self::AllowCoreUnloaders => "@rules.coreunloaders",
            Self::DerelictRepair => "@rules.derelictrepair",
            Self::ReactorExplosions => "@rules.reactorexplosions",
            Self::SchematicsAllowed => "@rules.schematic",
            Self::CoreIncinerates => "@rules.coreincinerates",
            Self::CleanupDeadTeams => "@rules.cleanupdeadteams",
            Self::DisableWorldProcessors => "@rules.disableworldprocessors",
            Self::HideBannedBlocks => "@rules.hidebannedblocks",
            Self::BlockWhitelist => "@bannedblocks.whitelist",
            Self::UnitCapVariable => "@rules.unitcapvariable",
            Self::UnitPayloadsExplode => "@rules.unitpayloadsexplode",
            Self::LogicUnitControl => "@rules.logicunitcontrol",
            Self::LogicUnitBuild => "@rules.logicunitbuild",
            Self::LogicUnitDeconstruct => "@rules.logicunitdeconstruct",
            Self::UnitWhitelist => "@bannedunits.whitelist",
            Self::AttackMode => "@rules.attack",
            Self::CoreCapture => "@rules.corecapture",
            Self::PlaceRangeCheck => "@rules.placerangecheck",
            Self::PolygonCoreProtection => "@rules.polygoncoreprotection",
            Self::PauseDisabled => "@rules.pauseDisabled",
            Self::DamageExplosions => "@rules.explosions",
            Self::Fire => "@rules.fire",
            Self::Fog => "@rules.fog",
            Self::Lighting => "@rules.lighting",
            Self::LimitMapArea => "@rules.limitarea",
            Self::AllowEditRules => "@rules.allowedit",
        }
    }

    pub fn value(self, rules: &Rules) -> bool {
        match self {
            Self::Waves => rules.waves,
            Self::WaveSending => rules.wave_sending,
            Self::WaveTimer => rules.wave_timer,
            Self::WaitEnemies => rules.wait_enemies,
            Self::RandomWaveAi => rules.random_wave_ai,
            Self::WavesSpawnAtCores => rules.waves_spawn_at_cores,
            Self::AirUseSpawns => rules.air_use_spawns,
            Self::AllowEditWorldProcessors => rules.allow_edit_world_processors,
            Self::InfiniteResources => rules.infinite_resources,
            Self::OnlyDepositCore => rules.only_deposit_core,
            Self::AllowCoreUnloaders => rules.allow_core_unloaders,
            Self::DerelictRepair => rules.derelict_repair,
            Self::ReactorExplosions => rules.reactor_explosions,
            Self::SchematicsAllowed => rules.schematics_allowed,
            Self::CoreIncinerates => rules.core_incinerates,
            Self::CleanupDeadTeams => rules.cleanup_dead_teams,
            Self::DisableWorldProcessors => rules.disable_world_processors,
            Self::HideBannedBlocks => rules.hide_banned_blocks,
            Self::BlockWhitelist => rules.block_whitelist,
            Self::UnitCapVariable => rules.unit_cap_variable,
            Self::UnitPayloadsExplode => rules.unit_payloads_explode,
            Self::LogicUnitControl => rules.logic_unit_control,
            Self::LogicUnitBuild => rules.logic_unit_build,
            Self::LogicUnitDeconstruct => rules.logic_unit_deconstruct,
            Self::UnitWhitelist => rules.unit_whitelist,
            Self::AttackMode => rules.attack_mode,
            Self::CoreCapture => rules.core_capture,
            Self::PlaceRangeCheck => rules.place_range_check,
            Self::PolygonCoreProtection => rules.polygon_core_protection,
            Self::PauseDisabled => rules.pause_disabled,
            Self::DamageExplosions => rules.damage_explosions,
            Self::Fire => rules.fire,
            Self::Fog => rules.fog,
            Self::Lighting => rules.lighting,
            Self::LimitMapArea => rules.limit_map_area,
            Self::AllowEditRules => rules.allow_edit_rules,
        }
    }

    pub fn enabled(self, rules: &Rules, context: &CustomRulesDialogContext) -> bool {
        match self {
            Self::WaveSending
            | Self::WaveTimer
            | Self::RandomWaveAi
            | Self::WavesSpawnAtCores
            | Self::AirUseSpawns => rules.waves,
            Self::WaitEnemies => rules.waves && rules.wave_timer,
            Self::CleanupDeadTeams => rules.pvp,
            Self::LogicUnitBuild | Self::LogicUnitDeconstruct => rules.logic_unit_control,
            Self::LimitMapArea => !context.state_is_game,
            _ => true,
        }
    }

    pub fn set(self, rules: &mut Rules, value: bool) {
        match self {
            Self::Waves => rules.waves = value,
            Self::WaveSending => rules.wave_sending = value,
            Self::WaveTimer => rules.wave_timer = value,
            Self::WaitEnemies => rules.wait_enemies = value,
            Self::RandomWaveAi => rules.random_wave_ai = value,
            Self::WavesSpawnAtCores => rules.waves_spawn_at_cores = value,
            Self::AirUseSpawns => rules.air_use_spawns = value,
            Self::AllowEditWorldProcessors => rules.allow_edit_world_processors = value,
            Self::InfiniteResources => rules.infinite_resources = value,
            Self::OnlyDepositCore => rules.only_deposit_core = value,
            Self::AllowCoreUnloaders => rules.allow_core_unloaders = value,
            Self::DerelictRepair => rules.derelict_repair = value,
            Self::ReactorExplosions => rules.reactor_explosions = value,
            Self::SchematicsAllowed => rules.schematics_allowed = value,
            Self::CoreIncinerates => rules.core_incinerates = value,
            Self::CleanupDeadTeams => rules.cleanup_dead_teams = value,
            Self::DisableWorldProcessors => rules.disable_world_processors = value,
            Self::HideBannedBlocks => rules.hide_banned_blocks = value,
            Self::BlockWhitelist => rules.block_whitelist = value,
            Self::UnitCapVariable => rules.unit_cap_variable = value,
            Self::UnitPayloadsExplode => rules.unit_payloads_explode = value,
            Self::LogicUnitControl => rules.logic_unit_control = value,
            Self::LogicUnitBuild => rules.logic_unit_build = value,
            Self::LogicUnitDeconstruct => rules.logic_unit_deconstruct = value,
            Self::UnitWhitelist => rules.unit_whitelist = value,
            Self::AttackMode => rules.attack_mode = value,
            Self::CoreCapture => rules.core_capture = value,
            Self::PlaceRangeCheck => rules.place_range_check = value,
            Self::PolygonCoreProtection => rules.polygon_core_protection = value,
            Self::PauseDisabled => rules.pause_disabled = value,
            Self::DamageExplosions => rules.damage_explosions = value,
            Self::Fire => rules.fire = value,
            Self::Fog => rules.fog = value,
            Self::Lighting => rules.lighting = value,
            Self::LimitMapArea => rules.limit_map_area = value,
            Self::AllowEditRules => rules.allow_edit_rules = value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesNumberKind {
    WaveSpacing,
    InitialWaveSpacing,
    DropZoneRadius,
    BuildCostMultiplier,
    BuildSpeedMultiplier,
    DeconstructRefundMultiplier,
    BlockHealthMultiplier,
    BlockDamageMultiplier,
    UnitDamageMultiplier,
    UnitCrashDamageMultiplier,
    UnitMineSpeedMultiplier,
    UnitBuildSpeedMultiplier,
    UnitCostMultiplier,
    EnemyCoreBuildRadius,
    SolarMultiplier,
}

impl CustomRulesNumberKind {
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::WaveSpacing => "@rules.wavespacing",
            Self::InitialWaveSpacing => "@rules.initialwavespacing",
            Self::DropZoneRadius => "@rules.dropzoneradius",
            Self::BuildCostMultiplier => "@rules.buildcostmultiplier",
            Self::BuildSpeedMultiplier => "@rules.buildspeedmultiplier",
            Self::DeconstructRefundMultiplier => "@rules.deconstructrefundmultiplier",
            Self::BlockHealthMultiplier => "@rules.blockhealthmultiplier",
            Self::BlockDamageMultiplier => "@rules.blockdamagemultiplier",
            Self::UnitDamageMultiplier => "@rules.unitdamagemultiplier",
            Self::UnitCrashDamageMultiplier => "@rules.unitcrashdamagemultiplier",
            Self::UnitMineSpeedMultiplier => "@rules.unitminespeedmultiplier",
            Self::UnitBuildSpeedMultiplier => "@rules.unitbuildspeedmultiplier",
            Self::UnitCostMultiplier => "@rules.unitcostmultiplier",
            Self::EnemyCoreBuildRadius => "@rules.enemycorebuildradius",
            Self::SolarMultiplier => "@rules.solarmultiplier",
        }
    }

    pub fn value(self, rules: &Rules) -> f32 {
        match self {
            Self::WaveSpacing => rules.wave_spacing / 60.0,
            Self::InitialWaveSpacing => rules.initial_wave_spacing / 60.0,
            Self::DropZoneRadius => rules.drop_zone_radius / TILE_SIZE as f32,
            Self::BuildCostMultiplier => rules.build_cost_multiplier,
            Self::BuildSpeedMultiplier => rules.build_speed_multiplier,
            Self::DeconstructRefundMultiplier => rules.deconstruct_refund_multiplier,
            Self::BlockHealthMultiplier => rules.block_health_multiplier,
            Self::BlockDamageMultiplier => rules.block_damage_multiplier,
            Self::UnitDamageMultiplier => rules.unit_damage_multiplier,
            Self::UnitCrashDamageMultiplier => rules.unit_crash_damage_multiplier,
            Self::UnitMineSpeedMultiplier => rules.unit_mine_speed_multiplier,
            Self::UnitBuildSpeedMultiplier => rules.unit_build_speed_multiplier,
            Self::UnitCostMultiplier => rules.unit_cost_multiplier,
            Self::EnemyCoreBuildRadius => {
                (rules.enemy_core_build_radius / TILE_SIZE as f32).min(200.0)
            }
            Self::SolarMultiplier => rules.solar_multiplier,
        }
    }

    pub fn bounds(self) -> (f32, f32) {
        match self {
            Self::WaveSpacing => (1.0, f32::MAX),
            Self::InitialWaveSpacing => (0.0, f32::MAX),
            Self::BuildSpeedMultiplier => (0.001, 50.0),
            Self::DeconstructRefundMultiplier => (0.0, 1.0),
            Self::UnitBuildSpeedMultiplier => (0.0, 50.0),
            _ => (0.0, f32::MAX),
        }
    }

    pub fn enabled(self, rules: &Rules) -> bool {
        match self {
            Self::WaveSpacing | Self::InitialWaveSpacing => rules.waves && rules.wave_timer,
            Self::DropZoneRadius => rules.waves,
            Self::BuildCostMultiplier | Self::DeconstructRefundMultiplier => {
                !rules.infinite_resources
            }
            Self::EnemyCoreBuildRadius => !rules.polygon_core_protection,
            _ => true,
        }
    }

    pub fn set(self, rules: &mut Rules, value: f32) {
        match self {
            Self::WaveSpacing => rules.wave_spacing = value * 60.0,
            Self::InitialWaveSpacing => rules.initial_wave_spacing = value * 60.0,
            Self::DropZoneRadius => rules.drop_zone_radius = value * TILE_SIZE as f32,
            Self::BuildCostMultiplier => rules.build_cost_multiplier = value,
            Self::BuildSpeedMultiplier => rules.build_speed_multiplier = value,
            Self::DeconstructRefundMultiplier => rules.deconstruct_refund_multiplier = value,
            Self::BlockHealthMultiplier => rules.block_health_multiplier = value,
            Self::BlockDamageMultiplier => rules.block_damage_multiplier = value,
            Self::UnitDamageMultiplier => rules.unit_damage_multiplier = value,
            Self::UnitCrashDamageMultiplier => rules.unit_crash_damage_multiplier = value,
            Self::UnitMineSpeedMultiplier => rules.unit_mine_speed_multiplier = value,
            Self::UnitBuildSpeedMultiplier => rules.unit_build_speed_multiplier = value,
            Self::UnitCostMultiplier => rules.unit_cost_multiplier = value,
            Self::EnemyCoreBuildRadius => rules.enemy_core_build_radius = value * TILE_SIZE as f32,
            Self::SolarMultiplier => rules.solar_multiplier = value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesIntegerKind {
    WinWave,
    UnitCap,
    LimitX,
    LimitY,
    LimitWidth,
    LimitHeight,
}

impl CustomRulesIntegerKind {
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::WinWave => "@rules.wavelimit",
            Self::UnitCap => "@rules.unitcap",
            Self::LimitX => "x",
            Self::LimitY => "y",
            Self::LimitWidth => "w",
            Self::LimitHeight => "h",
        }
    }

    pub fn value(self, rules: &Rules) -> i32 {
        match self {
            Self::WinWave => rules.win_wave,
            Self::UnitCap => rules.unit_cap,
            Self::LimitX => rules.limit_x,
            Self::LimitY => rules.limit_y,
            Self::LimitWidth => rules.limit_width,
            Self::LimitHeight => rules.limit_height,
        }
    }

    pub const fn bounds(self) -> (i32, i32) {
        match self {
            Self::WinWave => (0, i32::MAX),
            Self::UnitCap => (-999, 999),
            Self::LimitX | Self::LimitY | Self::LimitWidth | Self::LimitHeight => (0, 10_000),
        }
    }

    pub fn enabled(self, rules: &Rules, context: &CustomRulesDialogContext) -> bool {
        match self {
            Self::WinWave => rules.waves,
            Self::LimitX | Self::LimitY | Self::LimitWidth | Self::LimitHeight => {
                rules.limit_map_area && !context.state_is_game
            }
            Self::UnitCap => true,
        }
    }

    pub fn set(self, rules: &mut Rules, value: i32) {
        match self {
            Self::WinWave => rules.win_wave = value,
            Self::UnitCap => rules.unit_cap = value,
            Self::LimitX => rules.limit_x = value,
            Self::LimitY => rules.limit_y = value,
            Self::LimitWidth => rules.limit_width = value,
            Self::LimitHeight => rules.limit_height = value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesTeamSelectKind {
    DefaultTeam,
    WaveTeam,
}

impl CustomRulesTeamSelectKind {
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::DefaultTeam => "@rules.playerteam",
            Self::WaveTeam => "@rules.enemyteam",
        }
    }

    pub fn value(self, rules: &Rules) -> u8 {
        match self {
            Self::DefaultTeam => rules.default_team as u8,
            Self::WaveTeam => rules.wave_team as u8,
        }
    }

    pub fn set(self, rules: &mut Rules, team_id: u8) {
        match self {
            Self::DefaultTeam => rules.default_team = team_id as i32,
            Self::WaveTeam => rules.wave_team = team_id as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesTeamCheckKind {
    RtsAi,
    BuildAi,
    ProtectCores,
    CheckPlacement,
    InfiniteResources,
    FillItems,
}

impl CustomRulesTeamCheckKind {
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::RtsAi => "@rules.rtsai",
            Self::BuildAi => "@rules.buildai",
            Self::ProtectCores => "@rules.protectcores",
            Self::CheckPlacement => "@rules.checkplacement",
            Self::InfiniteResources => "@rules.infiniteresources",
            Self::FillItems => "@rules.fillitems",
        }
    }

    pub fn value(self, rule: &TeamRule) -> bool {
        match self {
            Self::RtsAi => rule.rts_ai,
            Self::BuildAi => rule.build_ai,
            Self::ProtectCores => rule.protect_cores,
            Self::CheckPlacement => rule.check_placement,
            Self::InfiniteResources => rule.infinite_resources,
            Self::FillItems => rule.fill_items,
        }
    }

    pub fn set(self, rule: &mut TeamRule, value: bool) {
        match self {
            Self::RtsAi => rule.rts_ai = value,
            Self::BuildAi => rule.build_ai = value,
            Self::ProtectCores => rule.protect_cores = value,
            Self::CheckPlacement => rule.check_placement = value,
            Self::InfiniteResources => rule.infinite_resources = value,
            Self::FillItems => rule.fill_items = value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesTeamNumberKind {
    BlockHealthMultiplier,
    BlockDamageMultiplier,
    RtsMinWeight,
    BuildAiTier,
    ExtraCoreBuildRadius,
    BuildSpeedMultiplier,
    UnitFactoryActivationDelay,
    UnitDamageMultiplier,
    UnitCrashDamageMultiplier,
    UnitMineSpeedMultiplier,
    UnitBuildSpeedMultiplier,
    UnitCostMultiplier,
    UnitHealthMultiplier,
}

impl CustomRulesTeamNumberKind {
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::BlockHealthMultiplier => "@rules.blockhealthmultiplier",
            Self::BlockDamageMultiplier => "@rules.blockdamagemultiplier",
            Self::RtsMinWeight => "@rules.rtsminattackweight",
            Self::BuildAiTier => "@rules.buildaitier",
            Self::ExtraCoreBuildRadius => "@rules.extracorebuildradius",
            Self::BuildSpeedMultiplier => "@rules.buildspeedmultiplier",
            Self::UnitFactoryActivationDelay => "@rules.unitfactoryactivation",
            Self::UnitDamageMultiplier => "@rules.unitdamagemultiplier",
            Self::UnitCrashDamageMultiplier => "@rules.unitcrashdamagemultiplier",
            Self::UnitMineSpeedMultiplier => "@rules.unitminespeedmultiplier",
            Self::UnitBuildSpeedMultiplier => "@rules.unitbuildspeedmultiplier",
            Self::UnitCostMultiplier => "@rules.unitcostmultiplier",
            Self::UnitHealthMultiplier => "@rules.unithealthmultiplier",
        }
    }

    pub fn value(self, rule: &TeamRule) -> f32 {
        match self {
            Self::BlockHealthMultiplier => rule.block_health_multiplier,
            Self::BlockDamageMultiplier => rule.block_damage_multiplier,
            Self::RtsMinWeight => rule.rts_min_weight,
            Self::BuildAiTier => rule.build_ai_tier,
            Self::ExtraCoreBuildRadius => rule.extra_core_build_radius / TILE_SIZE as f32,
            Self::BuildSpeedMultiplier => rule.build_speed_multiplier,
            Self::UnitFactoryActivationDelay => rule.unit_factory_activation_delay / 60.0,
            Self::UnitDamageMultiplier => rule.unit_damage_multiplier,
            Self::UnitCrashDamageMultiplier => rule.unit_crash_damage_multiplier,
            Self::UnitMineSpeedMultiplier => rule.unit_mine_speed_multiplier,
            Self::UnitBuildSpeedMultiplier => rule.unit_build_speed_multiplier,
            Self::UnitCostMultiplier => rule.unit_cost_multiplier,
            Self::UnitHealthMultiplier => rule.unit_health_multiplier,
        }
    }

    pub fn bounds(self) -> (f32, f32) {
        match self {
            Self::BuildAiTier => (0.0, 1.0),
            Self::BuildSpeedMultiplier | Self::UnitBuildSpeedMultiplier => (0.001, 50.0),
            _ => (0.0, f32::MAX),
        }
    }

    pub fn set(self, rule: &mut TeamRule, value: f32) {
        match self {
            Self::BlockHealthMultiplier => rule.block_health_multiplier = value,
            Self::BlockDamageMultiplier => rule.block_damage_multiplier = value,
            Self::RtsMinWeight => rule.rts_min_weight = value,
            Self::BuildAiTier => rule.build_ai_tier = value,
            Self::ExtraCoreBuildRadius => rule.extra_core_build_radius = value * TILE_SIZE as f32,
            Self::BuildSpeedMultiplier => rule.build_speed_multiplier = value,
            Self::UnitFactoryActivationDelay => rule.unit_factory_activation_delay = value * 60.0,
            Self::UnitDamageMultiplier => rule.unit_damage_multiplier = value,
            Self::UnitCrashDamageMultiplier => rule.unit_crash_damage_multiplier = value,
            Self::UnitMineSpeedMultiplier => rule.unit_mine_speed_multiplier = value,
            Self::UnitBuildSpeedMultiplier => rule.unit_build_speed_multiplier = value,
            Self::UnitCostMultiplier => rule.unit_cost_multiplier = value,
            Self::UnitHealthMultiplier => rule.unit_health_multiplier = value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesTeamIntegerKind {
    RtsMinSquad,
    RtsMaxSquad,
}

impl CustomRulesTeamIntegerKind {
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::RtsMinSquad => "@rules.rtsminsquadsize",
            Self::RtsMaxSquad => "@rules.rtsmaxsquadsize",
        }
    }

    pub fn value(self, rule: &TeamRule) -> i32 {
        match self {
            Self::RtsMinSquad => rule.rts_min_squad,
            Self::RtsMaxSquad => rule.rts_max_squad,
        }
    }

    pub const fn bounds(self) -> (i32, i32) {
        match self {
            Self::RtsMinSquad => (0, 100),
            Self::RtsMaxSquad => (1, 1000),
        }
    }

    pub fn set(self, rule: &mut TeamRule, value: i32) {
        match self {
            Self::RtsMinSquad => rule.rts_min_squad = value,
            Self::RtsMaxSquad => rule.rts_max_squad = value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomRulesTeamRuleKind {
    Check(CustomRulesTeamCheckKind),
    Number(CustomRulesTeamNumberKind),
    Integer(CustomRulesTeamIntegerKind),
}

impl CustomRulesTeamRuleKind {
    const JAVA_ORDER: [Self; 21] = [
        Self::Number(CustomRulesTeamNumberKind::BlockHealthMultiplier),
        Self::Number(CustomRulesTeamNumberKind::BlockDamageMultiplier),
        Self::Check(CustomRulesTeamCheckKind::RtsAi),
        Self::Integer(CustomRulesTeamIntegerKind::RtsMinSquad),
        Self::Integer(CustomRulesTeamIntegerKind::RtsMaxSquad),
        Self::Number(CustomRulesTeamNumberKind::RtsMinWeight),
        Self::Check(CustomRulesTeamCheckKind::BuildAi),
        Self::Number(CustomRulesTeamNumberKind::BuildAiTier),
        Self::Check(CustomRulesTeamCheckKind::ProtectCores),
        Self::Number(CustomRulesTeamNumberKind::ExtraCoreBuildRadius),
        Self::Check(CustomRulesTeamCheckKind::CheckPlacement),
        Self::Check(CustomRulesTeamCheckKind::InfiniteResources),
        Self::Check(CustomRulesTeamCheckKind::FillItems),
        Self::Number(CustomRulesTeamNumberKind::BuildSpeedMultiplier),
        Self::Number(CustomRulesTeamNumberKind::UnitFactoryActivationDelay),
        Self::Number(CustomRulesTeamNumberKind::UnitDamageMultiplier),
        Self::Number(CustomRulesTeamNumberKind::UnitCrashDamageMultiplier),
        Self::Number(CustomRulesTeamNumberKind::UnitMineSpeedMultiplier),
        Self::Number(CustomRulesTeamNumberKind::UnitBuildSpeedMultiplier),
        Self::Number(CustomRulesTeamNumberKind::UnitCostMultiplier),
        Self::Number(CustomRulesTeamNumberKind::UnitHealthMultiplier),
    ];

    fn label_key(self) -> &'static str {
        match self {
            Self::Check(kind) => kind.label_key(),
            Self::Number(kind) => kind.label_key(),
            Self::Integer(kind) => kind.label_key(),
        }
    }

    fn enabled(self, rules: &Rules, team_id: usize, team_rule: &TeamRule) -> bool {
        let erekir_default_env = Env::SCORCHING | Env::TERRESTRIAL;
        match self {
            Self::Check(CustomRulesTeamCheckKind::RtsAi) => team_id as i32 != rules.default_team,
            Self::Integer(CustomRulesTeamIntegerKind::RtsMinSquad)
            | Self::Integer(CustomRulesTeamIntegerKind::RtsMaxSquad)
            | Self::Number(CustomRulesTeamNumberKind::RtsMinWeight) => team_rule.rts_ai,
            Self::Check(CustomRulesTeamCheckKind::BuildAi) => {
                team_id as i32 != rules.default_team
                    && rules.env != erekir_default_env
                    && !rules.pvp
            }
            Self::Number(CustomRulesTeamNumberKind::BuildAiTier) => {
                team_rule.build_ai && rules.env != erekir_default_env && !rules.pvp
            }
            Self::Number(CustomRulesTeamNumberKind::ExtraCoreBuildRadius) => {
                !rules.polygon_core_protection && team_rule.protect_cores
            }
            _ => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesPlanet {
    pub name: String,
    pub localized_name: String,
    pub accessible: bool,
    pub visible: bool,
    pub landable: bool,
    pub default_env: u32,
}

impl CustomRulesPlanet {
    pub fn from_content(planet: &PlanetContent) -> Self {
        Self {
            name: planet.name().to_string(),
            localized_name: planet.localized_name().to_string(),
            accessible: planet.meta.accessible,
            visible: planet.meta.visible,
            landable: planet.is_landable(),
            default_env: planet.meta.default_env,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesWeather {
    pub name: String,
    pub localized_name: String,
    pub hidden: bool,
    pub duration: f32,
}

impl CustomRulesWeather {
    pub fn from_content(weather: &WeatherContent) -> Self {
        let weather = weather.weather();
        Self {
            name: weather.name().to_string(),
            localized_name: weather.localized_name().to_string(),
            hidden: weather.hidden,
            duration: weather.duration,
        }
    }

    pub fn to_entry(&self) -> WeatherEntry {
        let mut weather = Weather::new(0, self.name.clone());
        weather.duration = self.duration;
        WeatherEntry::new(&weather)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesContent {
    pub planets: Vec<CustomRulesPlanet>,
    pub weathers: Vec<CustomRulesWeather>,
    pub teams: TeamRegistry,
}

impl CustomRulesContent {
    pub fn vanilla() -> Self {
        Self {
            planets: crate::mindustry::content::planets::load()
                .iter()
                .map(CustomRulesPlanet::from_content)
                .collect(),
            weathers: crate::mindustry::content::weathers::load()
                .iter()
                .map(CustomRulesWeather::from_content)
                .collect(),
            teams: vanilla_teams(),
        }
    }

    pub fn accessible_planets(&self) -> Vec<&CustomRulesPlanet> {
        self.planets
            .iter()
            .filter(|planet| planet.accessible && planet.visible && planet.landable)
            .collect()
    }

    pub fn visible_weathers(&self) -> Vec<&CustomRulesWeather> {
        self.weathers
            .iter()
            .filter(|weather| !weather.hidden)
            .collect()
    }
}

impl Default for CustomRulesContent {
    fn default() -> Self {
        Self::vanilla()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomRulesAdditionalSetupEntry {
    pub category: CustomRulesCategory,
    pub label_key: String,
    pub tag_key: String,
    pub enabled_value: String,
    pub disabled_value: Option<String>,
}

impl CustomRulesAdditionalSetupEntry {
    pub fn tag_toggle(
        category: CustomRulesCategory,
        label_key: impl Into<String>,
        tag_key: impl Into<String>,
        enabled_value: impl Into<String>,
    ) -> Self {
        Self {
            category,
            label_key: label_key.into(),
            tag_key: tag_key.into(),
            enabled_value: enabled_value.into(),
            disabled_value: None,
        }
    }

    pub fn with_disabled_value(mut self, disabled_value: impl Into<String>) -> Self {
        self.disabled_value = Some(disabled_value.into());
        self
    }

    pub fn value(&self, rules: &Rules) -> bool {
        rules
            .tags
            .get(&self.tag_key)
            .map(|value| value == &self.enabled_value)
            .unwrap_or(false)
    }

    pub fn set(&self, rules: &mut Rules, value: bool) {
        if value {
            rules
                .tags
                .insert(self.tag_key.clone(), self.enabled_value.clone());
        } else if let Some(disabled_value) = &self.disabled_value {
            rules
                .tags
                .insert(self.tag_key.clone(), disabled_value.clone());
        } else {
            rules.tags.remove(&self.tag_key);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomRulesDialogAction {
    ShowDialog,
    Rebuild,
    RequestKeyboard,
    RequestScroll,
    ShowEditDialog,
    HideEditDialog,
    SetClipboardText(String),
    ShowInfoFade(&'static str),
    ShowErrorMessage(&'static str),
    OpenLoadout {
        capacity: i32,
        reset_item: ItemStack,
    },
    OpenBannedBlocks,
    OpenBannedUnits,
    OpenWeather,
    OpenColorPicker {
        color: [f32; 4],
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesEditDialogModel {
    pub title: &'static str,
    pub fill_parent: bool,
    pub close_button_added: bool,
    pub table_background: &'static str,
    pub button_style: &'static str,
    pub button_size: (f32, f32),
    pub margin_left: f32,
    pub buttons: Vec<CustomRulesEditDialogButton>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomRulesEditDialogButton {
    pub kind: CustomRulesEditDialogButtonKind,
    pub text: &'static str,
    pub icon: &'static str,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesEditDialogButtonKind {
    Copy,
    Load,
    Reset,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesWeatherDialogModel {
    pub title: &'static str,
    pub columns: usize,
    pub entries: Vec<CustomRulesWeatherEntryModel>,
    pub add_button_text: &'static str,
    pub add_button_icon: &'static str,
    pub add_button_width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesWeatherEntryModel {
    pub index: usize,
    pub weather: String,
    pub localized_name: String,
    pub width: f32,
    pub row: usize,
    pub column: usize,
    pub always: bool,
    pub fields_disabled: bool,
    pub numbers: Vec<CustomRulesWeatherNumberModel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesWeatherNumberModel {
    pub kind: CustomRulesWeatherNumberKind,
    pub label: &'static str,
    pub value_minutes: f32,
    pub value_text: String,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomRulesWeatherNumberKind {
    MinDuration,
    MaxDuration,
    MinFrequency,
    MaxFrequency,
}

impl CustomRulesWeatherNumberKind {
    pub const ALL: [Self; 4] = [
        Self::MinDuration,
        Self::MaxDuration,
        Self::MinFrequency,
        Self::MaxFrequency,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::MinDuration => "@rules.weather.duration.min",
            Self::MaxDuration => "@rules.weather.duration.max",
            Self::MinFrequency => "@rules.weather.frequency.min",
            Self::MaxFrequency => "@rules.weather.frequency.max",
        }
    }

    pub fn value_minutes(self, entry: &WeatherEntry) -> f32 {
        let ticks = match self {
            Self::MinDuration => entry.min_duration,
            Self::MaxDuration => entry.max_duration,
            Self::MinFrequency => entry.min_frequency,
            Self::MaxFrequency => entry.max_frequency,
        };
        ticks / WEATHER_MINUTE_TICKS
    }

    pub fn set_minutes(self, entry: &mut WeatherEntry, minutes: f32) {
        let value = minutes * WEATHER_MINUTE_TICKS;
        match self {
            Self::MinDuration => entry.min_duration = value,
            Self::MaxDuration => entry.max_duration = value,
            Self::MinFrequency => entry.min_frequency = value,
            Self::MaxFrequency => entry.max_frequency = value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesWeatherAddDialogModel {
    pub title: &'static str,
    pub background: &'static str,
    pub columns: usize,
    pub buttons: Vec<CustomRulesWeatherAddButton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesWeatherAddButton {
    pub index: usize,
    pub name: String,
    pub label: String,
    pub size: (f32, f32),
    pub row: usize,
    pub column: usize,
}

const WEATHER_MINUTE_TICKS: f32 = 60.0 * 60.0;

#[derive(Debug, Clone, PartialEq)]
pub struct CustomRulesDialog {
    pub rules: Rules,
    pub resetter: Rules,
    pub rule_search: String,
    pub show_rule_edit_rule: bool,
    pub additional_setup: Vec<CustomRulesAdditionalSetupEntry>,
    expanded_team_ids: BTreeSet<u8>,
}

impl Default for CustomRulesDialog {
    fn default() -> Self {
        Self::new(false)
    }
}

impl CustomRulesDialog {
    pub fn new(show_rule_edit_rule: bool) -> Self {
        Self {
            rules: Rules::default(),
            resetter: Rules::default(),
            rule_search: String::new(),
            show_rule_edit_rule,
            additional_setup: Vec::new(),
            expanded_team_ids: BTreeSet::new(),
        }
    }

    pub fn show(
        &mut self,
        rules: Rules,
        resetter: Rules,
        context: &CustomRulesDialogContext,
        content: &CustomRulesContent,
        locale: &str,
    ) -> CustomRulesDialogModel {
        self.rules = rules;
        self.resetter = resetter;
        self.rebuild(context, content, locale)
    }

    pub fn show_plan() -> Vec<CustomRulesDialogAction> {
        vec![CustomRulesDialogAction::ShowDialog]
    }

    pub fn refresh_plan(&self) -> Vec<CustomRulesDialogAction> {
        vec![
            CustomRulesDialogAction::Rebuild,
            CustomRulesDialogAction::RequestKeyboard,
            CustomRulesDialogAction::RequestScroll,
        ]
    }

    pub fn on_resize(
        &self,
        context: &CustomRulesDialogContext,
        content: &CustomRulesContent,
        locale: &str,
    ) -> (Vec<CustomRulesDialogAction>, CustomRulesDialogModel) {
        (
            vec![CustomRulesDialogAction::Rebuild],
            self.rebuild(context, content, locale),
        )
    }

    pub fn set_search(
        &mut self,
        text: &str,
        context: &CustomRulesDialogContext,
        content: &CustomRulesContent,
        locale: &str,
    ) -> CustomRulesDialogModel {
        self.rule_search = normalize_rule_search(text);
        self.rebuild(context, content, locale)
    }

    pub fn clear_search(
        &mut self,
        context: &CustomRulesDialogContext,
        content: &CustomRulesContent,
        locale: &str,
    ) -> CustomRulesDialogModel {
        self.rule_search.clear();
        self.rebuild(context, content, locale)
    }

    pub fn rebuild(
        &self,
        context: &CustomRulesDialogContext,
        content: &CustomRulesContent,
        locale: &str,
    ) -> CustomRulesDialogModel {
        CustomRulesDialogModel {
            title: CUSTOM_RULES_DIALOG_TITLE,
            fill_parent: true,
            close_button_added: true,
            edit_button: CustomRulesEditButton {
                text: CUSTOM_RULES_EDIT_BUTTON_TEXT,
                icon: CUSTOM_RULES_EDIT_BUTTON_ICON,
            },
            search: CustomRulesSearchModel {
                label: CUSTOM_RULES_SEARCH_LABEL,
                text: self.rule_search.clone(),
                clear_icon: CUSTOM_RULES_SEARCH_CLEAR_ICON,
                keyboard_focused: true,
            },
            scroll_x: context.graphics_width < 640.0,
            categories: self.sections(context, content, locale),
        }
    }

    pub fn edit_dialog_model(clipboard_text: Option<&str>) -> CustomRulesEditDialogModel {
        CustomRulesEditDialogModel {
            title: CUSTOM_RULES_EDIT_DIALOG_TITLE,
            fill_parent: false,
            close_button_added: true,
            table_background: CUSTOM_RULES_EDIT_TABLE_BACKGROUND,
            button_style: CUSTOM_RULES_EDIT_BUTTON_STYLE,
            button_size: CUSTOM_RULES_EDIT_BUTTON_SIZE,
            margin_left: CUSTOM_RULES_EDIT_BUTTON_MARGIN_LEFT,
            buttons: vec![
                CustomRulesEditDialogButton {
                    kind: CustomRulesEditDialogButtonKind::Copy,
                    text: "@waves.copy",
                    icon: "copy",
                    disabled: false,
                },
                CustomRulesEditDialogButton {
                    kind: CustomRulesEditDialogButtonKind::Load,
                    text: "@waves.load",
                    icon: "download",
                    disabled: clipboard_text.is_none_or(|text| !text.starts_with('{')),
                },
                CustomRulesEditDialogButton {
                    kind: CustomRulesEditDialogButtonKind::Reset,
                    text: "@settings.reset",
                    icon: "refresh",
                    disabled: false,
                },
            ],
        }
    }

    pub fn open_loadout_plan() -> Vec<CustomRulesDialogAction> {
        vec![CustomRulesDialogAction::OpenLoadout {
            capacity: CUSTOM_RULES_LOADOUT_CAPACITY,
            reset_item: ItemStack::new(
                CUSTOM_RULES_DEFAULT_LOADOUT_ITEM,
                CUSTOM_RULES_DEFAULT_LOADOUT_AMOUNT,
            ),
        }]
    }

    pub fn open_child_plan(action: CustomRulesButtonAction) -> Vec<CustomRulesDialogAction> {
        match action {
            CustomRulesButtonAction::OpenLoadout => Self::open_loadout_plan(),
            CustomRulesButtonAction::OpenBannedBlocks => {
                vec![CustomRulesDialogAction::OpenBannedBlocks]
            }
            CustomRulesButtonAction::OpenBannedUnits => {
                vec![CustomRulesDialogAction::OpenBannedUnits]
            }
            CustomRulesButtonAction::OpenWeather => vec![CustomRulesDialogAction::OpenWeather],
        }
    }

    pub fn open_ambient_light_plan(&self) -> Vec<CustomRulesDialogAction> {
        vec![CustomRulesDialogAction::OpenColorPicker {
            color: self.rules.ambient_light,
        }]
    }

    pub fn copy_rules_plan(&self) -> Vec<CustomRulesDialogAction> {
        vec![
            CustomRulesDialogAction::ShowInfoFade("@copied"),
            CustomRulesDialogAction::SetClipboardText(custom_rules_json_without_spawns(
                &self.rules,
            )),
            CustomRulesDialogAction::HideEditDialog,
        ]
    }

    pub fn load_rules_from_json_plan(&mut self, json: &str) -> Vec<CustomRulesDialogAction> {
        let old_spawns = self.rules.spawns.clone();
        let mut new_rules = self.rules.clone();
        match new_rules.apply_json_str(json) {
            Ok(()) => {
                new_rules.spawns = old_spawns;
                self.rules = new_rules;
                vec![
                    CustomRulesDialogAction::Rebuild,
                    CustomRulesDialogAction::HideEditDialog,
                ]
            }
            Err(_) => vec![
                CustomRulesDialogAction::ShowErrorMessage("@rules.invaliddata"),
                CustomRulesDialogAction::HideEditDialog,
            ],
        }
    }

    pub fn reset_rules_plan(&mut self) -> Vec<CustomRulesDialogAction> {
        self.rules = self.resetter.clone();
        vec![
            CustomRulesDialogAction::Rebuild,
            CustomRulesDialogAction::HideEditDialog,
        ]
    }

    pub fn set_check(&mut self, kind: CustomRulesCheckKind, value: bool) {
        kind.set(&mut self.rules, value);
    }

    pub fn set_number(&mut self, kind: CustomRulesNumberKind, value: f32) {
        kind.set(&mut self.rules, value);
    }

    pub fn set_integer(&mut self, kind: CustomRulesIntegerKind, value: i32) {
        kind.set(&mut self.rules, value);
    }

    pub fn select_team(&mut self, kind: CustomRulesTeamSelectKind, team_id: u8) {
        kind.set(&mut self.rules, team_id);
    }

    pub fn toggle_team_section(&mut self, team_id: u8) {
        if !self.expanded_team_ids.remove(&team_id) {
            self.expanded_team_ids.insert(team_id);
        }
    }

    pub fn set_team_check(&mut self, team_id: usize, kind: CustomRulesTeamCheckKind, value: bool) {
        kind.set(self.rules.teams.get_or_insert(team_id), value);
    }

    pub fn set_team_number(&mut self, team_id: usize, kind: CustomRulesTeamNumberKind, value: f32) {
        kind.set(self.rules.teams.get_or_insert(team_id), value);
    }

    pub fn set_team_integer(
        &mut self,
        team_id: usize,
        kind: CustomRulesTeamIntegerKind,
        value: i32,
    ) {
        kind.set(self.rules.teams.get_or_insert(team_id), value);
    }

    pub fn set_additional_check(&mut self, index: usize, value: bool) {
        let entry = &self.additional_setup[index];
        entry.set(&mut self.rules, value);
    }

    pub fn set_ambient_light(&mut self, color: [f32; 4]) {
        self.rules.ambient_light = color;
    }

    pub fn select_planet(&mut self, planet: &CustomRulesPlanet) {
        apply_custom_game_planet_rules(&mut self.rules, planet);
    }

    pub fn select_any_env(&mut self) {
        self.rules.env = Env::ANY;
        self.rules.planet = "sun".into();
    }

    pub fn weather_dialog_model(
        &mut self,
        context: &CustomRulesDialogContext,
        content: &CustomRulesContent,
    ) -> CustomRulesWeatherDialogModel {
        self.rules.weather.retain(|entry| {
            !entry.weather.is_empty()
                && content
                    .weathers
                    .iter()
                    .any(|weather| weather.name == entry.weather)
        });
        let columns = weather_columns(context.graphics_width, context.scl);
        let entries = self
            .rules
            .weather
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let localized_name = content
                    .weathers
                    .iter()
                    .find(|weather| weather.name == entry.weather)
                    .map(|weather| weather.localized_name.clone())
                    .unwrap_or_else(|| entry.weather.clone());
                CustomRulesWeatherEntryModel {
                    index,
                    weather: entry.weather.clone(),
                    localized_name,
                    width: CUSTOM_RULES_WEATHER_CARD_WIDTH,
                    row: index / columns,
                    column: index % columns,
                    always: entry.always,
                    fields_disabled: entry.always,
                    numbers: CustomRulesWeatherNumberKind::ALL
                        .iter()
                        .copied()
                        .map(|kind| CustomRulesWeatherNumberModel {
                            kind,
                            label: kind.label(),
                            value_minutes: kind.value_minutes(entry),
                            value_text: format_custom_rules_number(kind.value_minutes(entry)),
                            disabled: entry.always,
                        })
                        .collect(),
                }
            })
            .collect();
        CustomRulesWeatherDialogModel {
            title: "@rules.weather",
            columns,
            entries,
            add_button_text: "@add",
            add_button_icon: "add",
            add_button_width: 170.0,
        }
    }

    pub fn weather_add_dialog_model(
        content: &CustomRulesContent,
    ) -> CustomRulesWeatherAddDialogModel {
        CustomRulesWeatherAddDialogModel {
            title: "@add",
            background: "Tex.button",
            columns: CUSTOM_RULES_WEATHER_ADD_COLUMNS,
            buttons: content
                .visible_weathers()
                .into_iter()
                .enumerate()
                .map(|(index, weather)| CustomRulesWeatherAddButton {
                    index,
                    name: weather.name.clone(),
                    label: weather.localized_name.clone(),
                    size: CUSTOM_RULES_WEATHER_ADD_BUTTON_SIZE,
                    row: index / CUSTOM_RULES_WEATHER_ADD_COLUMNS,
                    column: index % CUSTOM_RULES_WEATHER_ADD_COLUMNS,
                })
                .collect(),
        }
    }

    pub fn add_weather(&mut self, weather: &CustomRulesWeather) {
        self.rules.weather.push(weather.to_entry());
    }

    pub fn remove_weather(&mut self, index: usize) {
        self.rules.weather.remove(index);
    }

    pub fn set_weather_always(&mut self, index: usize, value: bool) {
        self.rules.weather[index].always = value;
    }

    pub fn set_weather_number(
        &mut self,
        index: usize,
        kind: CustomRulesWeatherNumberKind,
        minutes: f32,
    ) {
        kind.set_minutes(&mut self.rules.weather[index], minutes);
    }

    fn sections(
        &self,
        context: &CustomRulesDialogContext,
        content: &CustomRulesContent,
        locale: &str,
    ) -> Vec<CustomRulesSection> {
        let mut sections = CustomRulesCategory::JAVA_ORDER
            .iter()
            .copied()
            .map(|category| CustomRulesSection {
                category,
                name: category.name(),
                title_key: category.title_key(),
                title: bundle_value(locale, &category.title_key()),
                rows: Vec::new(),
            })
            .collect::<Vec<_>>();

        self.populate_waves(&mut sections[0], context, locale);
        self.populate_resources(&mut sections[1], context, locale);
        self.populate_unit(&mut sections[2], context, locale);
        self.populate_enemy(&mut sections[3], context, locale);
        self.populate_environment(&mut sections[4], context, locale);
        self.populate_planet(&mut sections[5], content, locale);
        self.populate_teams(&mut sections[6], context, content, locale);
        self.populate_additional_setup(&mut sections, context, locale);

        sections
            .into_iter()
            .filter(|section| !section.rows.is_empty())
            .collect()
    }

    fn populate_waves(
        &self,
        section: &mut CustomRulesSection,
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        for kind in [
            CustomRulesCheckKind::Waves,
            CustomRulesCheckKind::WaveSending,
            CustomRulesCheckKind::WaveTimer,
            CustomRulesCheckKind::WaitEnemies,
            CustomRulesCheckKind::RandomWaveAi,
            CustomRulesCheckKind::WavesSpawnAtCores,
            CustomRulesCheckKind::AirUseSpawns,
        ] {
            self.add_check(section, kind, context, locale);
        }
        self.add_integer(section, CustomRulesIntegerKind::WinWave, context, locale);
        for kind in [
            CustomRulesNumberKind::WaveSpacing,
            CustomRulesNumberKind::InitialWaveSpacing,
            CustomRulesNumberKind::DropZoneRadius,
        ] {
            self.add_number(section, kind, locale);
        }
    }

    fn populate_resources(
        &self,
        section: &mut CustomRulesSection,
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        for kind in [
            CustomRulesCheckKind::AllowEditWorldProcessors,
            CustomRulesCheckKind::InfiniteResources,
            CustomRulesCheckKind::OnlyDepositCore,
            CustomRulesCheckKind::AllowCoreUnloaders,
            CustomRulesCheckKind::DerelictRepair,
            CustomRulesCheckKind::ReactorExplosions,
            CustomRulesCheckKind::SchematicsAllowed,
            CustomRulesCheckKind::CoreIncinerates,
            CustomRulesCheckKind::CleanupDeadTeams,
            CustomRulesCheckKind::DisableWorldProcessors,
        ] {
            self.add_check(section, kind, context, locale);
        }
        for kind in [
            CustomRulesNumberKind::BuildCostMultiplier,
            CustomRulesNumberKind::BuildSpeedMultiplier,
            CustomRulesNumberKind::DeconstructRefundMultiplier,
            CustomRulesNumberKind::BlockHealthMultiplier,
            CustomRulesNumberKind::BlockDamageMultiplier,
        ] {
            self.add_number(section, kind, locale);
        }
        self.add_button_if_matches(
            section,
            "configure",
            "@configure",
            CustomRulesButtonAction::OpenLoadout,
            CUSTOM_RULES_RULE_BUTTON_WIDTH,
            locale,
        );
        self.add_button_if_matches(
            section,
            "bannedblocks",
            "@bannedblocks",
            CustomRulesButtonAction::OpenBannedBlocks,
            CUSTOM_RULES_RULE_BUTTON_WIDTH,
            locale,
        );
        self.add_check(
            section,
            CustomRulesCheckKind::HideBannedBlocks,
            context,
            locale,
        );
        self.add_check(
            section,
            CustomRulesCheckKind::BlockWhitelist,
            context,
            locale,
        );
    }

    fn populate_unit(
        &self,
        section: &mut CustomRulesSection,
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        for kind in [
            CustomRulesCheckKind::UnitCapVariable,
            CustomRulesCheckKind::UnitPayloadsExplode,
        ] {
            self.add_check(section, kind, context, locale);
        }
        self.add_integer(section, CustomRulesIntegerKind::UnitCap, context, locale);
        for kind in [
            CustomRulesNumberKind::UnitDamageMultiplier,
            CustomRulesNumberKind::UnitCrashDamageMultiplier,
            CustomRulesNumberKind::UnitMineSpeedMultiplier,
            CustomRulesNumberKind::UnitBuildSpeedMultiplier,
            CustomRulesNumberKind::UnitCostMultiplier,
        ] {
            self.add_number(section, kind, locale);
        }
        for kind in [
            CustomRulesCheckKind::LogicUnitControl,
            CustomRulesCheckKind::LogicUnitBuild,
            CustomRulesCheckKind::LogicUnitDeconstruct,
        ] {
            self.add_check(section, kind, context, locale);
        }
        self.add_button_if_matches(
            section,
            "bannedunits",
            "@bannedunits",
            CustomRulesButtonAction::OpenBannedUnits,
            CUSTOM_RULES_RULE_BUTTON_WIDTH,
            locale,
        );
        self.add_check(
            section,
            CustomRulesCheckKind::UnitWhitelist,
            context,
            locale,
        );
    }

    fn populate_enemy(
        &self,
        section: &mut CustomRulesSection,
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        for kind in [
            CustomRulesCheckKind::AttackMode,
            CustomRulesCheckKind::CoreCapture,
            CustomRulesCheckKind::PlaceRangeCheck,
            CustomRulesCheckKind::PolygonCoreProtection,
        ] {
            self.add_check(section, kind, context, locale);
        }
        self.add_number(section, CustomRulesNumberKind::EnemyCoreBuildRadius, locale);
    }

    fn populate_environment(
        &self,
        section: &mut CustomRulesSection,
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        for kind in [
            CustomRulesCheckKind::PauseDisabled,
            CustomRulesCheckKind::DamageExplosions,
            CustomRulesCheckKind::Fire,
            CustomRulesCheckKind::Fog,
            CustomRulesCheckKind::Lighting,
            CustomRulesCheckKind::LimitMapArea,
        ] {
            self.add_check(section, kind, context, locale);
        }
        for kind in [
            CustomRulesIntegerKind::LimitX,
            CustomRulesIntegerKind::LimitY,
            CustomRulesIntegerKind::LimitWidth,
            CustomRulesIntegerKind::LimitHeight,
        ] {
            self.add_integer(section, kind, context, locale);
        }
        self.add_number(section, CustomRulesNumberKind::SolarMultiplier, locale);
        if matches_bundle_key(locale, "rules.ambientlight", &self.rule_search) {
            section.rows.push(CustomRulesRow {
                label_key: "@rules.ambientlight".into(),
                label: bundle_value(locale, "rules.ambientlight"),
                enabled: true,
                info: None,
                kind: CustomRulesRowKind::Color {
                    kind: CustomRulesColorKind::AmbientLight,
                    rgba: self.rules.ambient_light,
                    width: CUSTOM_RULES_SMALL_BUTTON_WIDTH,
                },
            });
        }
        self.add_button_if_matches(
            section,
            "rules.weather",
            "@rules.weather",
            CustomRulesButtonAction::OpenWeather,
            CUSTOM_RULES_SMALL_BUTTON_WIDTH,
            locale,
        );
    }

    fn populate_planet(
        &self,
        section: &mut CustomRulesSection,
        content: &CustomRulesContent,
        locale: &str,
    ) {
        if !matches_bundle_key(locale, "rules.title.planet", &self.rule_search) {
            return;
        }
        let options = content
            .accessible_planets()
            .into_iter()
            .enumerate()
            .map(|(index, planet)| CustomRulesPlanetButton {
                index,
                name: planet.name.clone(),
                label: planet.localized_name.clone(),
                env: planet.default_env,
                checked: self.rules.planet == planet.name,
                row: index / CUSTOM_RULES_PLANET_COLUMNS,
                column: index % CUSTOM_RULES_PLANET_COLUMNS,
            })
            .collect();
        section.rows.push(CustomRulesRow {
            label_key: "@rules.title.planet".into(),
            label: bundle_value(locale, "rules.title.planet"),
            enabled: true,
            info: None,
            kind: CustomRulesRowKind::PlanetSelector(CustomRulesPlanetSelectorModel {
                background: CUSTOM_RULES_PLANET_TABLE_BACKGROUND,
                button_style: CUSTOM_RULES_PLANET_BUTTON_STYLE,
                button_size: CUSTOM_RULES_PLANET_BUTTON_SIZE,
                columns: CUSTOM_RULES_PLANET_COLUMNS,
                options,
                any_env: CustomRulesPlanetButton {
                    index: usize::MAX,
                    name: "sun".into(),
                    label: bundle_value(locale, "rules.anyenv"),
                    env: Env::ANY,
                    checked: self.rules.planet == "sun",
                    row: 0,
                    column: 0,
                },
            }),
        });
    }

    fn populate_teams(
        &self,
        section: &mut CustomRulesSection,
        context: &CustomRulesDialogContext,
        content: &CustomRulesContent,
        locale: &str,
    ) {
        if self.show_rule_edit_rule {
            self.add_check(
                section,
                CustomRulesCheckKind::AllowEditRules,
                context,
                locale,
            );
        }
        self.add_team_select(
            section,
            CustomRulesTeamSelectKind::DefaultTeam,
            content,
            locale,
        );
        self.add_team_select(
            section,
            CustomRulesTeamSelectKind::WaveTeam,
            content,
            locale,
        );
        for team in content.teams.base_teams() {
            self.add_team_section(section, team, context, locale);
        }
    }

    fn populate_additional_setup(
        &self,
        sections: &mut [CustomRulesSection],
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        for (index, entry) in self.additional_setup.iter().enumerate() {
            if !label_matches(locale, &entry.label_key, &self.rule_search) {
                continue;
            }
            if let Some(section) = sections
                .iter_mut()
                .find(|section| section.category == entry.category)
            {
                section.rows.push(CustomRulesRow {
                    label_key: entry.label_key.clone(),
                    label: label_text(locale, &entry.label_key),
                    enabled: true,
                    info: rule_info(locale, &entry.label_key, context),
                    kind: CustomRulesRowKind::AdditionalCheck {
                        index,
                        tag_key: entry.tag_key.clone(),
                        checked: entry.value(&self.rules),
                    },
                });
            }
        }
    }

    fn add_check(
        &self,
        section: &mut CustomRulesSection,
        kind: CustomRulesCheckKind,
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        let label_key = kind.label_key();
        if !label_matches(locale, label_key, &self.rule_search) {
            return;
        }
        section.rows.push(CustomRulesRow {
            label_key: label_key.into(),
            label: label_text(locale, label_key),
            enabled: kind.enabled(&self.rules, context),
            info: rule_info(locale, label_key, context),
            kind: CustomRulesRowKind::Check {
                kind,
                checked: kind.value(&self.rules),
            },
        });
    }

    fn add_number(
        &self,
        section: &mut CustomRulesSection,
        kind: CustomRulesNumberKind,
        locale: &str,
    ) {
        let label_key = kind.label_key();
        if !label_matches(locale, label_key, &self.rule_search) {
            return;
        }
        let (min, max) = kind.bounds();
        let value = kind.value(&self.rules);
        section.rows.push(CustomRulesRow {
            label_key: label_key.into(),
            label: label_text(locale, label_key),
            enabled: kind.enabled(&self.rules),
            info: rule_info(locale, label_key, &CustomRulesDialogContext::default()),
            kind: CustomRulesRowKind::Number {
                kind,
                value,
                value_text: format_custom_rules_number(value),
                min,
                max,
            },
        });
    }

    fn add_integer(
        &self,
        section: &mut CustomRulesSection,
        kind: CustomRulesIntegerKind,
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        let label_key = kind.label_key();
        if !label_matches(locale, label_key, &self.rule_search) {
            return;
        }
        let (min, max) = kind.bounds();
        section.rows.push(CustomRulesRow {
            label_key: label_key.into(),
            label: label_text(locale, label_key),
            enabled: kind.enabled(&self.rules, context),
            info: rule_info(locale, label_key, context),
            kind: CustomRulesRowKind::Integer {
                kind,
                value: kind.value(&self.rules),
                min,
                max,
            },
        });
    }

    fn add_button_if_matches(
        &self,
        section: &mut CustomRulesSection,
        search_key: &str,
        label_key: &str,
        action: CustomRulesButtonAction,
        width: f32,
        locale: &str,
    ) {
        if matches_bundle_key(locale, search_key, &self.rule_search) {
            section.rows.push(CustomRulesRow {
                label_key: label_key.into(),
                label: label_text(locale, label_key),
                enabled: true,
                info: None,
                kind: CustomRulesRowKind::Button { action, width },
            });
        }
    }

    fn add_team_select(
        &self,
        section: &mut CustomRulesSection,
        kind: CustomRulesTeamSelectKind,
        content: &CustomRulesContent,
        locale: &str,
    ) {
        let label_key = kind.label_key();
        if !label_matches(locale, label_key, &self.rule_search) {
            return;
        }
        let selected_team = kind.value(&self.rules);
        let buttons = content
            .teams
            .base_teams()
            .iter()
            .map(|team| {
                team_button(
                    team,
                    selected_team == team.id,
                    (CUSTOM_RULES_TEAM_BUTTON_SIZE, CUSTOM_RULES_TEAM_BUTTON_SIZE),
                )
            })
            .collect();
        section.rows.push(CustomRulesRow {
            label_key: label_key.into(),
            label: label_text(locale, label_key),
            enabled: true,
            info: None,
            kind: CustomRulesRowKind::TeamSelect {
                kind,
                selected_team,
                buttons,
            },
        });
    }

    fn add_team_section(
        &self,
        section: &mut CustomRulesSection,
        team: &Team,
        context: &CustomRulesDialogContext,
        locale: &str,
    ) {
        let team_rule = self.rules.teams.get_or_default(team.id as usize);
        let mut rows = Vec::new();
        for kind in CustomRulesTeamRuleKind::JAVA_ORDER {
            let label_key = kind.label_key();
            if !label_matches(locale, label_key, &self.rule_search) {
                continue;
            }
            let enabled = kind.enabled(&self.rules, team.id as usize, &team_rule);
            let field = match kind {
                CustomRulesTeamRuleKind::Check(kind) => CustomRulesTeamRuleField::Check {
                    kind,
                    checked: kind.value(&team_rule),
                },
                CustomRulesTeamRuleKind::Number(kind) => {
                    let (min, max) = kind.bounds();
                    let value = kind.value(&team_rule);
                    CustomRulesTeamRuleField::Number {
                        kind,
                        value,
                        value_text: format_custom_rules_number(value),
                        min,
                        max,
                    }
                }
                CustomRulesTeamRuleKind::Integer(kind) => {
                    let (min, max) = kind.bounds();
                    CustomRulesTeamRuleField::Integer {
                        kind,
                        value: kind.value(&team_rule),
                        min,
                        max,
                    }
                }
            };
            rows.push(CustomRulesTeamRuleRow {
                label_key: label_key.into(),
                label: label_text(locale, label_key),
                enabled,
                info: rule_info(locale, label_key, context),
                field,
            });
        }
        if rows.is_empty() {
            return;
        }
        section.rows.push(CustomRulesRow {
            label_key: team.name.clone(),
            label: team.colored_name_token(),
            enabled: true,
            info: None,
            kind: CustomRulesRowKind::TeamRules(CustomRulesTeamSection {
                team: team_button(team, false, CUSTOM_RULES_TEAM_SECTION_BUTTON_SIZE),
                expanded: self.expanded_team_ids.contains(&team.id),
                button_size: CUSTOM_RULES_TEAM_SECTION_BUTTON_SIZE,
                rows,
            }),
        });
    }
}

pub fn normalize_rule_search(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn weather_columns(graphics_width: f32, scl: f32) -> usize {
    let scl = if scl == 0.0 { 1.0 } else { scl };
    ((graphics_width / (CUSTOM_RULES_WEATHER_COLUMN_WIDTH * scl)) as usize).max(1)
}

pub fn format_custom_rules_number(value: f32) -> String {
    if (value - value.round()).abs() < 0.01 {
        format!("{}", value.round() as i32)
    } else {
        format!("{value:.1}")
    }
}

pub fn custom_rules_json_without_spawns(rules: &Rules) -> String {
    let mut pairs = Vec::new();
    macro_rules! pair {
        ($key:literal, $value:expr) => {
            pairs.push(format!("\"{}\":{}", $key, $value));
        };
    }
    pair!("waves", rules.waves);
    pair!("waveTimer", rules.wave_timer);
    pair!("waveSending", rules.wave_sending);
    pair!("showSpawns", rules.show_spawns);
    pair!("waitEnemies", rules.wait_enemies);
    pair!("randomWaveAI", rules.random_wave_ai);
    pair!("wavesSpawnAtCores", rules.waves_spawn_at_cores);
    pair!("airUseSpawns", rules.air_use_spawns);
    pair!("attackMode", rules.attack_mode);
    pair!("pvp", rules.pvp);
    pair!("editor", rules.editor);
    pair!("infiniteResources", rules.infinite_resources);
    pair!("allowEditRules", rules.allow_edit_rules);
    pair!(
        "allowEditWorldProcessors",
        rules.allow_edit_world_processors
    );
    pair!("disableWorldProcessors", rules.disable_world_processors);
    pair!("onlyDepositCore", rules.only_deposit_core);
    pair!("allowCoreUnloaders", rules.allow_core_unloaders);
    pair!("derelictRepair", rules.derelict_repair);
    pair!("reactorExplosions", rules.reactor_explosions);
    pair!("cleanupDeadTeams", rules.cleanup_dead_teams);
    pair!("fog", rules.fog);
    pair!("staticFog", rules.static_fog);
    pair!("lighting", rules.lighting);
    pair!("coreIncinerates", rules.core_incinerates);
    pair!("coreDestroyClear", rules.core_destroy_clear);
    pair!("borderDarkness", rules.border_darkness);
    pair!("limitMapArea", rules.limit_map_area);
    pair!("limitX", rules.limit_x);
    pair!("limitY", rules.limit_y);
    pair!("limitWidth", rules.limit_width);
    pair!("limitHeight", rules.limit_height);
    pair!("disableOutsideArea", rules.disable_outside_area);
    pair!(
        "customBackgroundCallback",
        json_optional_string(rules.custom_background_callback.as_deref())
    );
    pair!(
        "backgroundTexture",
        json_optional_string(rules.background_texture.as_deref())
    );
    pair!("backgroundSpeed", rules.background_speed);
    pair!("backgroundScl", rules.background_scl);
    pair!("backgroundOffsetX", rules.background_offset_x);
    pair!("backgroundOffsetY", rules.background_offset_y);
    pair!("allowLogicData", rules.allow_logic_data);
    pair!("schematicsAllowed", rules.schematics_allowed);
    pair!("hideBannedBlocks", rules.hide_banned_blocks);
    pair!("blockWhitelist", rules.block_whitelist);
    pair!("unitWhitelist", rules.unit_whitelist);
    pair!("unitPayloadsExplode", rules.unit_payloads_explode);
    pair!("unitCapVariable", rules.unit_cap_variable);
    pair!("logicUnitControl", rules.logic_unit_control);
    pair!("logicUnitBuild", rules.logic_unit_build);
    pair!("logicUnitDeconstruct", rules.logic_unit_deconstruct);
    pair!("coreCapture", rules.core_capture);
    pair!("placeRangeCheck", rules.place_range_check);
    pair!("polygonCoreProtection", rules.polygon_core_protection);
    pair!("pauseDisabled", rules.pause_disabled);
    pair!("damageExplosions", rules.damage_explosions);
    pair!("fire", rules.fire);
    pair!(
        "bannedBlocks",
        json_string_array_from_set(&rules.banned_blocks)
    );
    pair!(
        "bannedUnits",
        json_string_array_from_set(&rules.banned_units)
    );
    pair!("loadout", json_item_stacks(&rules.loadout));
    pair!("weather", json_weather_entries(&rules.weather));
    pair!("ambientLight", json_f32_array4(rules.ambient_light));
    pair!("itemDepositCooldown", rules.item_deposit_cooldown);
    pair!("dragMultiplier", rules.drag_multiplier);
    pair!("waveSpacing", rules.wave_spacing);
    pair!("initialWaveSpacing", rules.initial_wave_spacing);
    pair!("dropZoneRadius", rules.drop_zone_radius);
    pair!("winWave", rules.win_wave);
    pair!("unitCap", rules.unit_cap);
    pair!("enemyCoreBuildRadius", rules.enemy_core_build_radius);
    pair!("solarMultiplier", rules.solar_multiplier);
    pair!("buildCostMultiplier", rules.build_cost_multiplier);
    pair!("buildSpeedMultiplier", rules.build_speed_multiplier);
    pair!(
        "deconstructRefundMultiplier",
        rules.deconstruct_refund_multiplier
    );
    pair!("blockHealthMultiplier", rules.block_health_multiplier);
    pair!("blockDamageMultiplier", rules.block_damage_multiplier);
    pair!(
        "unitBuildSpeedMultiplier",
        rules.unit_build_speed_multiplier
    );
    pair!("unitCostMultiplier", rules.unit_cost_multiplier);
    pair!("unitDamageMultiplier", rules.unit_damage_multiplier);
    pair!("unitHealthMultiplier", rules.unit_health_multiplier);
    pair!(
        "unitCrashDamageMultiplier",
        rules.unit_crash_damage_multiplier
    );
    pair!("unitMineSpeedMultiplier", rules.unit_mine_speed_multiplier);
    pair!("defaultTeam", rules.default_team);
    pair!("waveTeam", rules.wave_team);
    pair!("modeName", json_optional_string(rules.mode_name.as_deref()));
    pair!("planet", json_string(&rules.planet));
    pair!("env", rules.env);
    pair!("tags", json_string_map(&rules.tags));
    pair!("teams", json_team_rules(rules));
    format!("{{{}}}", pairs.join(","))
}

fn apply_custom_game_planet_rules(rules: &mut Rules, planet: &CustomRulesPlanet) {
    rules.env = planet.default_env;
    rules.planet = planet.name.clone();
    match planet.name.as_str() {
        "erekir" => {
            rules.wave_team = TEAM_MALIS as i32;
            rules.place_range_check = false;
            rules.show_spawns = true;
            rules.fog = true;
            rules.static_fog = true;
            rules.lighting = false;
            rules.core_destroy_clear = true;
            rules.only_deposit_core = true;
        }
        "serpulo" => {
            rules.wave_team = TEAM_CRUX as i32;
            rules.place_range_check = false;
            rules.show_spawns = false;
            rules.core_destroy_clear = true;
        }
        _ => {}
    }
}

fn team_button(team: &Team, checked: bool, size: (f32, f32)) -> CustomRulesTeamButton {
    CustomRulesTeamButton {
        id: team.id,
        name: team.name.clone(),
        color_rgba: team.color_rgba,
        colored_name: team.colored_name_token(),
        checked,
        size,
    }
}

fn bundle_key(label_key: &str) -> &str {
    label_key.strip_prefix('@').unwrap_or(label_key)
}

fn bundle_value(locale: &str, key: &str) -> String {
    upstream_menu_bundle_value_for_locale(locale, key)
        .unwrap_or(key)
        .to_string()
}

fn label_text(locale: &str, label_key: &str) -> String {
    if label_key.starts_with('@') {
        bundle_value(locale, bundle_key(label_key))
    } else {
        label_key.to_string()
    }
}

fn label_matches(locale: &str, label_key: &str, search: &str) -> bool {
    if search.is_empty() {
        return true;
    }
    label_key.starts_with('@') && matches_bundle_key(locale, bundle_key(label_key), search)
}

fn matches_bundle_key(locale: &str, key: &str, search: &str) -> bool {
    search.is_empty()
        || upstream_menu_bundle_value_for_locale(locale, key)
            .unwrap_or(key)
            .to_lowercase()
            .contains(search)
}

fn rule_info(
    locale: &str,
    label_key: &str,
    context: &CustomRulesDialogContext,
) -> Option<CustomRulesInfo> {
    if !label_key.starts_with('@') {
        return None;
    }
    let key = format!("{}.info", bundle_key(label_key));
    upstream_menu_bundle_value_for_locale(locale, &key).map(|_| CustomRulesInfo {
        key: format!("{label_key}.info"),
        presentation: if context.mobile && !context.portrait {
            CustomRulesInfoPresentation::MobileInfoButton
        } else {
            CustomRulesInfoPresentation::Tooltip
        },
    })
}

fn json_escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape_string(value))
}

fn json_optional_string(value: Option<&str>) -> String {
    value.map(json_string).unwrap_or_else(|| "null".into())
}

fn json_string_array_from_set(values: &BTreeSet<String>) -> String {
    let values = values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>();
    format!("[{}]", values.join(","))
}

fn json_string_map(values: &BTreeMap<String, String>) -> String {
    let values = values
        .iter()
        .map(|(key, value)| format!("{}:{}", json_string(key), json_string(value)))
        .collect::<Vec<_>>();
    format!("{{{}}}", values.join(","))
}

fn json_f32_array4(values: [f32; 4]) -> String {
    format!(
        "[{:.4},{:.4},{:.4},{:.4}]",
        values[0], values[1], values[2], values[3]
    )
}

fn json_item_stacks(values: &[ItemStack]) -> String {
    let values = values
        .iter()
        .map(|stack| {
            format!(
                "{{\"item\":{},\"amount\":{}}}",
                json_string(&stack.item),
                stack.amount
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", values.join(","))
}

fn json_weather_entries(values: &[WeatherEntry]) -> String {
    let values = values
        .iter()
        .map(|entry| {
            format!(
                concat!(
                    "{{",
                    "\"weather\":{},",
                    "\"minFrequency\":{},",
                    "\"maxFrequency\":{},",
                    "\"minDuration\":{},",
                    "\"maxDuration\":{},",
                    "\"cooldown\":{},",
                    "\"intensity\":{},",
                    "\"always\":{}",
                    "}}"
                ),
                json_string(&entry.weather),
                entry.min_frequency,
                entry.max_frequency,
                entry.min_duration,
                entry.max_duration,
                entry.cooldown,
                entry.intensity,
                entry.always
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", values.join(","))
}

fn json_team_rule(rule: &TeamRule) -> String {
    format!(
        concat!(
            "{{",
            "\"aiCoreSpawn\":{},",
            "\"protectCores\":{},",
            "\"checkPlacement\":{},",
            "\"cheat\":{},",
            "\"fillItems\":{},",
            "\"infiniteResources\":{},",
            "\"infiniteAmmo\":{},",
            "\"prebuildAi\":{},",
            "\"buildAi\":{},",
            "\"buildAiTier\":{},",
            "\"respawn\":{},",
            "\"unitDamageMultiplier\":{},",
            "\"unitHealthMultiplier\":{},",
            "\"unitCrashDamageMultiplier\":{},",
            "\"unitMineSpeedMultiplier\":{},",
            "\"unitCostMultiplier\":{},",
            "\"unitBuildSpeedMultiplier\":{},",
            "\"blockDamageMultiplier\":{},",
            "\"blockHealthMultiplier\":{},",
            "\"buildSpeedMultiplier\":{},",
            "\"rtsAi\":{},",
            "\"rtsMinSquad\":{},",
            "\"rtsMaxSquad\":{},",
            "\"rtsMinWeight\":{},",
            "\"unitFactoryActivationDelay\":{},",
            "\"extraCoreBuildRadius\":{}",
            "}}"
        ),
        rule.ai_core_spawn,
        rule.protect_cores,
        rule.check_placement,
        rule.cheat,
        rule.fill_items,
        rule.infinite_resources,
        rule.infinite_ammo,
        rule.prebuild_ai,
        rule.build_ai,
        rule.build_ai_tier,
        rule.respawn,
        rule.unit_damage_multiplier,
        rule.unit_health_multiplier,
        rule.unit_crash_damage_multiplier,
        rule.unit_mine_speed_multiplier,
        rule.unit_cost_multiplier,
        rule.unit_build_speed_multiplier,
        rule.block_damage_multiplier,
        rule.block_health_multiplier,
        rule.build_speed_multiplier,
        rule.rts_ai,
        rule.rts_min_squad,
        rule.rts_max_squad,
        rule.rts_min_weight,
        rule.unit_factory_activation_delay,
        rule.extra_core_build_radius
    )
}

fn json_team_rules(rules: &Rules) -> String {
    let values = rules
        .teams
        .iter_present()
        .map(|(team_id, rule)| {
            format!(
                "{}:{}",
                json_string(&team_id.to_string()),
                json_team_rule(rule)
            )
        })
        .collect::<Vec<_>>();
    format!("{{{}}}", values.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        game::{SpawnGroup, TEAM_CRUX, TEAM_MALIS, TEAM_SHARDED},
        world::meta::Env,
    };

    fn context() -> CustomRulesDialogContext {
        CustomRulesDialogContext {
            mobile: false,
            portrait: false,
            graphics_width: 1280.0,
            scl: 1.0,
            state_is_game: false,
        }
    }

    fn content() -> CustomRulesContent {
        CustomRulesContent::vanilla()
    }

    #[test]
    fn show_builds_java_category_order_and_core_rows() {
        let content = content();
        let mut dialog = CustomRulesDialog::new(true);
        let model = dialog.show(
            Rules::default(),
            Rules::default(),
            &context(),
            &content,
            "en",
        );

        assert_eq!(model.title, CUSTOM_RULES_DIALOG_TITLE);
        assert!(model.fill_parent);
        assert_eq!(
            model
                .categories
                .iter()
                .map(|section| section.category)
                .collect::<Vec<_>>(),
            CustomRulesCategory::JAVA_ORDER
        );
        assert!(matches!(
            model.categories[0].rows[0].kind,
            CustomRulesRowKind::Check {
                kind: CustomRulesCheckKind::Waves,
                ..
            }
        ));
        assert!(model.categories[1].rows.iter().any(|row| matches!(
            row.kind,
            CustomRulesRowKind::Button {
                action: CustomRulesButtonAction::OpenLoadout,
                ..
            }
        )));
        assert!(model.categories[6].rows.iter().any(|row| matches!(
            row.kind,
            CustomRulesRowKind::Check {
                kind: CustomRulesCheckKind::AllowEditRules,
                ..
            }
        )));
    }

    #[test]
    fn search_normalizes_and_filters_empty_sections_like_java_setup_main() {
        let content = content();
        let mut dialog = CustomRulesDialog::new(false);
        let model = dialog.set_search("   wave    spacing  ", &context(), &content, "en");

        assert_eq!(dialog.rule_search, "wave spacing");
        assert_eq!(model.categories.len(), 1);
        assert_eq!(model.categories[0].category, CustomRulesCategory::Waves);
        assert!(model.categories[0].rows.iter().all(|row| {
            row.label.to_lowercase().contains("wave spacing")
                || row.label.to_lowercase().contains("initial wave spacing")
        }));

        let cleared = dialog.clear_search(&context(), &content, "en");
        assert!(dialog.rule_search.is_empty());
        assert_eq!(
            cleared.categories.len(),
            CustomRulesCategory::JAVA_ORDER.len()
        );
    }

    #[test]
    fn conditions_and_unit_conversions_follow_java_fields() {
        let content = content();
        let mut rules = Rules::default();
        rules.waves = false;
        rules.infinite_resources = true;
        rules.logic_unit_control = false;
        rules.pvp = false;
        rules.limit_map_area = true;
        let mut game_context = context();
        game_context.state_is_game = true;

        let mut dialog = CustomRulesDialog::new(false);
        let model = dialog.show(rules, Rules::default(), &game_context, &content, "en");
        let waves = &model.categories[0];
        let spacing = waves
            .rows
            .iter()
            .find(|row| {
                matches!(
                    row.kind,
                    CustomRulesRowKind::Number {
                        kind: CustomRulesNumberKind::WaveSpacing,
                        ..
                    }
                )
            })
            .unwrap();
        assert!(!spacing.enabled);

        let resources = &model.categories[1];
        let build_cost = resources
            .rows
            .iter()
            .find(|row| {
                matches!(
                    row.kind,
                    CustomRulesRowKind::Number {
                        kind: CustomRulesNumberKind::BuildCostMultiplier,
                        ..
                    }
                )
            })
            .unwrap();
        assert!(!build_cost.enabled);

        let environment = &model.categories[4];
        let limit_area = environment
            .rows
            .iter()
            .find(|row| {
                matches!(
                    row.kind,
                    CustomRulesRowKind::Check {
                        kind: CustomRulesCheckKind::LimitMapArea,
                        ..
                    }
                )
            })
            .unwrap();
        assert!(!limit_area.enabled);

        dialog.set_number(CustomRulesNumberKind::WaveSpacing, 90.0);
        dialog.set_number(CustomRulesNumberKind::DropZoneRadius, 7.0);
        assert_eq!(dialog.rules.wave_spacing, 90.0 * 60.0);
        assert_eq!(dialog.rules.drop_zone_radius, 7.0 * TILE_SIZE as f32);
    }

    #[test]
    fn edit_dialog_copy_load_and_reset_preserve_java_side_effect_order() {
        let mut dialog = CustomRulesDialog::new(false);
        dialog.rules.waves = true;
        dialog.rules.only_deposit_core = true;
        dialog.rules.spawns.push(SpawnGroup::default());
        dialog.rules.loadout = vec![ItemStack::new("lead", 25)];
        dialog.resetter.waves = false;
        dialog.resetter.only_deposit_core = false;

        let edit = CustomRulesDialog::edit_dialog_model(Some("{\"waves\":false}"));
        assert!(!edit.buttons[1].disabled);
        assert!(CustomRulesDialog::edit_dialog_model(Some("[]")).buttons[1].disabled);

        let copy = dialog.copy_rules_plan();
        let CustomRulesDialogAction::SetClipboardText(json) = &copy[1] else {
            panic!("missing clipboard action");
        };
        assert!(!json.contains("\"spawns\""));
        assert!(json.contains("\"onlyDepositCore\":true"));
        assert!(json.contains("\"loadout\":[{\"item\":\"lead\",\"amount\":25}]"));

        let actions = dialog.load_rules_from_json_plan(
            r#"{"waves":false,"onlyDepositCore":false,"loadout":[{"item":"copper","amount":100}]}"#,
        );
        assert_eq!(
            actions,
            vec![
                CustomRulesDialogAction::Rebuild,
                CustomRulesDialogAction::HideEditDialog
            ]
        );
        assert!(!dialog.rules.waves);
        assert!(!dialog.rules.only_deposit_core);
        assert_eq!(dialog.rules.spawns.len(), 1);
        assert_eq!(dialog.rules.loadout, vec![ItemStack::new("copper", 100)]);

        let reset = dialog.reset_rules_plan();
        assert_eq!(
            reset,
            vec![
                CustomRulesDialogAction::Rebuild,
                CustomRulesDialogAction::HideEditDialog
            ]
        );
        assert!(!dialog.rules.only_deposit_core);
    }

    #[test]
    fn planet_selector_filters_accessible_visible_landable_and_applies_rule_setters() {
        let content = content();
        let mut dialog = CustomRulesDialog::new(false);
        let model = dialog.show(
            Rules::default(),
            Rules::default(),
            &context(),
            &content,
            "en",
        );
        let planet_row = model.categories[5]
            .rows
            .iter()
            .find_map(|row| match &row.kind {
                CustomRulesRowKind::PlanetSelector(selector) => Some(selector),
                _ => None,
            })
            .unwrap();
        let names = planet_row
            .options
            .iter()
            .map(|button| button.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["erekir", "serpulo"]);

        let erekir = content
            .planets
            .iter()
            .find(|planet| planet.name == "erekir")
            .unwrap();
        dialog.select_planet(erekir);
        assert_eq!(dialog.rules.planet, "erekir");
        assert_eq!(dialog.rules.env, Env::SCORCHING | Env::TERRESTRIAL);
        assert_eq!(dialog.rules.wave_team, TEAM_MALIS as i32);
        assert!(dialog.rules.only_deposit_core);
        assert!(dialog.rules.fog);

        let serpulo = content
            .planets
            .iter()
            .find(|planet| planet.name == "serpulo")
            .unwrap();
        dialog.select_planet(serpulo);
        assert_eq!(dialog.rules.wave_team, TEAM_CRUX as i32);
        assert!(!dialog.rules.show_spawns);

        dialog.select_any_env();
        assert_eq!(dialog.rules.planet, "sun");
        assert_eq!(dialog.rules.env, Env::ANY);
    }

    #[test]
    fn weather_dialog_columns_entries_add_remove_and_always_flags_match_java_flow() {
        let content = content();
        let mut dialog = CustomRulesDialog::new(false);
        let rain = content
            .weathers
            .iter()
            .find(|weather| weather.name == "rain")
            .unwrap();
        let hidden = content
            .weathers
            .iter()
            .find(|weather| weather.hidden)
            .unwrap();
        dialog.add_weather(rain);
        dialog.rules.weather.push(WeatherEntry {
            weather: hidden.name.clone(),
            ..WeatherEntry::default()
        });

        let mut small = context();
        small.graphics_width = 800.0;
        let model = dialog.weather_dialog_model(&small, &content);
        assert_eq!(model.columns, 1);
        assert_eq!(model.entries.len(), 2);
        assert_eq!(model.entries[0].weather, "rain");

        dialog.set_weather_always(0, true);
        let model = dialog.weather_dialog_model(&small, &content);
        assert!(model.entries[0].fields_disabled);
        assert!(model.entries[0]
            .numbers
            .iter()
            .all(|number| number.disabled));
        dialog.set_weather_number(0, CustomRulesWeatherNumberKind::MinDuration, 3.0);
        assert_eq!(
            dialog.rules.weather[0].min_duration,
            3.0 * WEATHER_MINUTE_TICKS
        );
        dialog.remove_weather(0);
        assert_eq!(dialog.rules.weather[0].weather, hidden.name);

        let add = CustomRulesDialog::weather_add_dialog_model(&content);
        assert!(add.buttons.iter().all(|button| button.name != hidden.name));
        assert_eq!(add.columns, CUSTOM_RULES_WEATHER_ADD_COLUMNS);
    }

    #[test]
    fn team_selectors_and_team_rule_conditions_use_base_teams() {
        let content = content();
        let mut rules = Rules::default();
        rules.default_team = TEAM_SHARDED as i32;
        rules.env = Env::SCORCHING | Env::TERRESTRIAL;
        rules.pvp = false;
        let mut dialog = CustomRulesDialog::new(false);
        let model = dialog.show(rules, Rules::default(), &context(), &content, "en");
        let teams = &model.categories[6];
        let team_select = teams
            .rows
            .iter()
            .find_map(|row| match &row.kind {
                CustomRulesRowKind::TeamSelect {
                    kind: CustomRulesTeamSelectKind::DefaultTeam,
                    buttons,
                    ..
                } => Some(buttons),
                _ => None,
            })
            .unwrap();
        assert_eq!(team_select.len(), 6);
        assert!(team_select[TEAM_SHARDED as usize].checked);

        let sharded_section = teams
            .rows
            .iter()
            .find_map(|row| match &row.kind {
                CustomRulesRowKind::TeamRules(section) if section.team.id == TEAM_SHARDED => {
                    Some(section)
                }
                _ => None,
            })
            .unwrap();
        let rts = sharded_section
            .rows
            .iter()
            .find(|row| {
                matches!(
                    row.field,
                    CustomRulesTeamRuleField::Check {
                        kind: CustomRulesTeamCheckKind::RtsAi,
                        ..
                    }
                )
            })
            .unwrap();
        let build_ai = sharded_section
            .rows
            .iter()
            .find(|row| {
                matches!(
                    row.field,
                    CustomRulesTeamRuleField::Check {
                        kind: CustomRulesTeamCheckKind::BuildAi,
                        ..
                    }
                )
            })
            .unwrap();
        assert!(!rts.enabled);
        assert!(!build_ai.enabled);

        dialog.select_team(CustomRulesTeamSelectKind::DefaultTeam, TEAM_CRUX);
        assert_eq!(dialog.rules.default_team, TEAM_CRUX as i32);
    }

    #[test]
    fn rule_info_uses_tooltip_on_desktop_and_button_on_mobile_landscape() {
        let content = content();
        let mut dialog = CustomRulesDialog::new(false);
        let desktop = dialog.show(
            Rules::default(),
            Rules::default(),
            &context(),
            &content,
            "en",
        );
        let waves_spawn_info = desktop.categories[0]
            .rows
            .iter()
            .find(|row| {
                matches!(
                    row.kind,
                    CustomRulesRowKind::Check {
                        kind: CustomRulesCheckKind::WavesSpawnAtCores,
                        ..
                    }
                )
            })
            .unwrap()
            .info
            .as_ref()
            .unwrap();
        assert_eq!(
            waves_spawn_info.presentation,
            CustomRulesInfoPresentation::Tooltip
        );

        let mut mobile = context();
        mobile.mobile = true;
        mobile.portrait = false;
        let mobile_model = dialog.rebuild(&mobile, &content, "en");
        let mobile_info = mobile_model.categories[0]
            .rows
            .iter()
            .find(|row| {
                matches!(
                    row.kind,
                    CustomRulesRowKind::Check {
                        kind: CustomRulesCheckKind::WavesSpawnAtCores,
                        ..
                    }
                )
            })
            .unwrap()
            .info
            .as_ref()
            .unwrap();
        assert_eq!(
            mobile_info.presentation,
            CustomRulesInfoPresentation::MobileInfoButton
        );
    }

    #[test]
    fn additional_setup_appends_modded_tag_toggles_into_requested_category() {
        let content = content();
        let mut dialog = CustomRulesDialog::new(false);
        dialog
            .additional_setup
            .push(CustomRulesAdditionalSetupEntry::tag_toggle(
                CustomRulesCategory::Enemy,
                "@rules.attack",
                "modded-rule",
                "enabled",
            ));
        dialog.set_additional_check(0, true);
        let model = dialog.rebuild(&context(), &content, "en");
        assert_eq!(dialog.rules.tags.get("modded-rule").unwrap(), "enabled");
        assert!(model.categories[3].rows.iter().any(|row| matches!(
            row.kind,
            CustomRulesRowKind::AdditionalCheck {
                index: 0,
                checked: true,
                ..
            }
        )));
    }
}
