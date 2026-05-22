pub mod abilities;
pub mod comp;
pub mod damage;
pub mod entity_group;
pub mod entity_indexer;
pub mod fires;
pub mod leg;
pub mod leg_destroy_data;
pub mod mover;
pub mod pattern;
pub mod predict;
pub mod puddles;
pub mod sized;
pub mod target_priority;
pub mod unit_sorts;
pub mod units;

pub use abilities::{
    Ability, BasicAbility, LiquidExplodeAbility, LiquidRegenAbility, RegenAbility,
    SpawnDeathAbility,
};
pub use comp::BuildingComp;
pub use comp::WeaponsComp;
pub use comp::{apply_armor, ShieldComp};
pub use comp::{BlockUnitBuilding, BlockUnitComp};
pub use comp::{
    BuilderBlockInfo, BuilderComp, BuilderRequirement, BuilderSkipContext, BuilderTileSnapshot,
};
pub use comp::{BuildingTetherAction, BuildingTetherComp, BuildingTetherRef};
pub use comp::{
    BulletComp, BulletSpec, ChildComp, ChildParent, DamageComp, DamageState, DrawComp,
    ElevationMoveComp, Interval, OwnerComp, PosComp, PosTeamDef, RotComp, ShielderComp, SolidPred,
    TeamState, TimerComp,
};
pub use comp::{CrawlComp, CrawlSolidPred, CrawlType, CrawlUpdateInput};
pub use comp::{DecalColor, DecalComp, DecalDrawPlan, DecalRegion};
pub use comp::{EffectRenderInput, EffectStateComp};
pub use comp::{EntityComp, EntityIoState, EntityLocality};
pub use comp::{FireComp, FireTile, FireUpdateContext, FireUpdatePlan};
pub use comp::{HealthComp, ItemStackSlot, ItemsComp};
pub use comp::{HitboxComp, HitboxRect};
pub use comp::{LaunchCoreBlock, LaunchCoreComp, LaunchCoreDrawPlan, LaunchCoreSmoke};
pub use comp::{LegsComp, LegsSolidPred, LegsType, LegsUpdateInput};
pub use comp::{MechComp, MechStepPlan, MechType};
pub use comp::{MineItem, MineTile, MinerComp, MinerType, MinerUpdateContext, MinerUpdatePlan};
pub use comp::{PayloadComp, PayloadKind, PayloadState};
pub use comp::{PhysicRef, PhysicsComp};
pub use comp::{PlayerComp, PlayerUnitState};
pub use comp::{PowerGraphUpdate, PowerGraphUpdaterComp};
pub use comp::{PuddleComp, PuddleLiquid, PuddleTile, PuddleUpdateContext, PuddleUpdatePlan};
pub use comp::{SegmentComp, SegmentRef, SegmentType};
pub use comp::{StatusColor, StatusComp};
pub use comp::{SyncComp, SyncHooks};
pub use comp::{TankComp, TankType, TankUpdateInput, TankUpdatePlan};
pub use comp::{TeamComp, TeamRulesView, VelComp};
pub use comp::{TimedComp, TimedKillComp};
pub use comp::{TrailState, WaterMoveComp, WaterMoveDrawPlan};
pub use comp::{UnderwaterDrawPlan, UnderwaterMoveComp};
pub use comp::{
    UnitCollisionLayer, UnitComp, UnitControllerState, UnitFloorSnapshot, UnitTrailState,
};
pub use comp::{UnitTetherAction, UnitTetherComp, UnitTetherRef};
pub use comp::{WaterCrawlComp, WaterCrawlSolidPred};
pub use comp::{WorldLabelAlign, WorldLabelComp, WorldLabelDrawPlan};
pub use damage::{
    calculate_damage, complete_damage_tiles, find_length, pierce_result_length,
    tile_damage_edge_scaled_damage, tile_damage_ray_count, DAMAGE_FALLOFF,
};
pub use entity_group::{EntityGroup, EntityGroupItem, Rect, SpatialEntity};
pub use entity_indexer::EntityIndexer;
pub use fires::{ExtinguishResult, FireCreateResult, FireRules, Fires, BASE_FIRE_LIFETIME};
pub use leg::Leg;
pub use leg_destroy_data::{LegDestroyData, TextureRegionRef};
pub use mover::Mover;
pub use pattern::{
    BulletHandler, ShootAlternate, ShootBarrel, ShootHelix, ShootPattern, ShootSine, ShootSpread,
    ShootSummon, Shot,
};
pub use predict::{intercept, intercept_positions};
pub use puddles::{
    react_puddle, PuddleDepositContext, PuddleDepositOutcome, PuddleDepositResult, PuddleEntry,
    PuddleLiquidInfo, PuddleReactionResult, PuddleTileView, Puddles, MAX_LIQUID,
};
pub use sized::{EntityPosition, SizedEntity};
pub use target_priority as TargetPriority;
pub use unit_sorts::{
    building_default, building_water, closest, farthest, strongest, weakest,
    BuildingPriorityTarget, SortTarget,
};
pub use units::{BuildPlan, StatusEntry, WeaponMount};

// Mirrors upstream core/src/mindustry/entities. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.
