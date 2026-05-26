//! Incremental Rust mirror of upstream `mindustry.entities.comp`.

pub mod block_unit;
pub mod builder;
pub mod building;
pub mod building_tether;
pub mod bullet;
pub mod child;
pub mod crawl;
pub mod damage;
pub mod decal;
pub mod draw;
pub mod effect_state;
pub mod elevation_move;
pub mod entity;
pub mod fire;
pub mod health;
pub mod hitbox;
pub mod items;
pub mod launch_core;
pub mod legs;
pub mod mech;
pub mod miner;
pub mod owner;
pub mod payload;
pub mod physics;
pub mod player;
pub mod pos;
pub mod pos_team_def;
pub mod power_graph_updater;
pub mod puddle;
pub mod rot;
pub mod segment;
pub mod shield;
pub mod shielder;
pub mod status;
pub mod sync;
pub mod tank;
pub mod team;
pub mod timed;
pub mod timed_kill;
pub mod timer;
pub mod underwater_move;
pub mod unit;
pub mod unit_tether;
pub mod vel;
pub mod water_crawl;
pub mod water_move;
pub mod weapons;
pub mod world_label;

pub use block_unit::{BlockUnitBuilding, BlockUnitComp};
pub use builder::{
    BuilderAiAssistCandidate, BuilderAiFollowCandidate, BuilderAiFollowSearch,
    BuilderAiMoveToAssist, BuilderAiMoveToPlan, BuilderAiRuntimeBranch, BuilderAiRuntimeInput,
    BuilderAiRuntimeState, BuilderAiRuntimeStep, BuilderBlockInfo, BuilderComp, BuilderRequirement,
    BuilderSkipContext, BuilderTileSnapshot, PrebuildAiRuntimeInput, PrebuildAiRuntimeState,
    PrebuildAiRuntimeStep, BUILDER_AI_ASSIST_WITHIN_EXTRA_RANGE,
};
pub use building::BuildingComp;
pub use building_tether::{BuildingTetherAction, BuildingTetherComp, BuildingTetherRef};
pub use bullet::{BulletComp, BulletPropValue, BulletSpec};
pub use child::{ChildComp, ChildParent};
pub use crawl::{CrawlComp, CrawlSolidPred, CrawlType, CrawlUpdateInput};
pub use damage::DamageComp;
pub use decal::{DecalColor, DecalComp, DecalDrawPlan, DecalRegion};
pub use draw::DrawComp;
pub use effect_state::{EffectRenderInput, EffectStateComp};
pub use elevation_move::{ElevationMoveComp, SolidPred};
pub use entity::{EntityComp, EntityIoState, EntityLocality};
pub use fire::{FireComp, FireTile, FireUpdateContext, FireUpdatePlan};
pub use health::HealthComp;
pub use hitbox::{HitboxComp, HitboxRect};
pub use items::{ItemStackSlot, ItemsComp};
pub use launch_core::{LaunchCoreBlock, LaunchCoreComp, LaunchCoreDrawPlan, LaunchCoreSmoke};
pub use legs::{LegsComp, LegsSolidPred, LegsType, LegsUpdateInput};
pub use mech::{MechComp, MechStepPlan, MechType};
pub use miner::{
    apply_prebuild_mining_tick, MineItem, MineTile, MinerComp, MinerType, MinerUpdateContext,
    MinerUpdatePlan, PrebuildMiningRuntimeStep,
};
pub use owner::OwnerComp;
pub use payload::{PayloadComp, PayloadKind, PayloadState};
pub use physics::{PhysicRef, PhysicsComp};
pub use player::{PlayerComp, PlayerUnitState, PlayerUnitSwitchContext};
pub use pos::PosComp;
pub use pos_team_def::PosTeamDef;
pub use power_graph_updater::{PowerGraphUpdate, PowerGraphUpdaterComp};
pub use puddle::{PuddleComp, PuddleLiquid, PuddleTile, PuddleUpdateContext, PuddleUpdatePlan};
pub use rot::RotComp;
pub use segment::{SegmentComp, SegmentRef, SegmentType};
pub use shield::{apply_armor, ShieldComp};
pub use shielder::{DamageState, ShielderComp, TeamState};
pub use status::{StatusColor, StatusComp};
pub use sync::{SyncComp, SyncHooks};
pub use tank::{TankComp, TankType, TankUpdateInput, TankUpdatePlan};
pub use team::{TeamComp, TeamRulesView};
pub use timed::TimedComp;
pub use timed_kill::TimedKillComp;
pub use timer::{BuildingTimerState, Interval, TimerComp};
pub use underwater_move::{UnderwaterDrawPlan, UnderwaterMoveComp};
pub use unit::{
    PrebuildAiUnitInput, UnitCollisionLayer, UnitComp, UnitControllerState, UnitFloorSnapshot,
    UnitTrailState,
};
pub use unit_tether::{UnitTetherAction, UnitTetherComp, UnitTetherRef};
pub use vel::VelComp;
pub use water_crawl::{WaterCrawlComp, WaterCrawlSolidPred};
pub use water_move::{TrailState, WaterMoveComp, WaterMoveDrawPlan};
pub use weapons::WeaponsComp;
pub use world_label::{WorldLabelAlign, WorldLabelComp, WorldLabelDrawPlan};
