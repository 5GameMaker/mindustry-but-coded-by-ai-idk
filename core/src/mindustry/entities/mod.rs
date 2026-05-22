pub mod abilities;
pub mod bullet;
pub mod comp;
pub mod damage;
pub mod effect;
pub mod entity_collisions;
pub mod entity_group;
pub mod entity_indexer;
pub mod fires;
pub mod group_defs;
pub mod leg;
pub mod leg_destroy_data;
pub mod lightning;
pub mod mover;
pub mod part;
pub mod pattern;
pub mod predict;
pub mod puddles;
pub mod sized;
pub mod target_priority;
pub mod unit_sorts;
pub mod units;

pub use abilities::{
    Ability, ArmorPlateAbility, ArmorPlateUpdate, BasicAbility, EnergyFieldAbility,
    EnergyFieldAction, EnergyFieldHit, EnergyFieldPulse, EnergyFieldTarget, ForceFieldAbility,
    ForceFieldHit, ForceFieldUpdate, LiquidExplodeAbility, LiquidRegenAbility, MoveEffectAbility,
    MoveEffectPlan, MoveLightningAbility, MoveLightningPlan, RegenAbility, RepairFieldAbility,
    RepairFieldPulse, RepairFieldTarget, ShieldArcAbility, ShieldArcHit, ShieldArcHitAction,
    ShieldArcUpdate, ShieldRegenFieldAbility, ShieldRegenFieldPulse, ShieldRegenFieldTarget,
    SpawnDeathAbility, StatusFieldAbility, StatusFieldPulse, SuppressionFieldAbility,
    SuppressionFieldPulse, UnitSpawnAbility, UnitSpawnPlan,
};
pub use bullet::{
    bomb_bullet_type, empty_bullet_type, explosion_bullet_type, missile_bullet_type,
    ArtilleryBulletType, ArtilleryTrailPlan, BasicBulletDrawPlan, BasicBulletType,
    BulletCreatePlan, BulletType, ContinuousBulletType, ContinuousDamagePlan,
    ContinuousFlameBulletType, ContinuousLaserBulletType, EmpBulletType, EmpEnemyPowerPlan,
    EmpFriendlyBuildingPlan, EmpUnitHitPlan, FireBulletType, FireBulletUpdatePlan, FlakBulletType,
    FlakUpdatePlan, InterceptorBulletType, InterceptorHitPlan, LaserBoltBulletType,
    LaserBoltDrawPlan, LaserBulletType, LaserDrawPlan, LaserInitPlan, LaserLayerDrawPlan,
    LaserLightningPlan, LightningBulletType, LiquidBulletType, LiquidDrawPlan, LiquidHitPlan,
    LiquidUpdatePlan, MassDriverBolt, MassDriverDropPlan, MassDriverExplosionPlan,
    MassDriverUpdatePlan, MultiBulletCreatePlan, MultiBulletType, PointBulletType,
    PointLaserBulletType, PointLaserUpdatePlan, RailBulletType, RailInitPlan, RailPiercePlan,
    RailPointEffectPlan, SapBulletType, SapDataPlan, SapDrawPlan, SapInitPlan, SapTargetInfo,
    SapTargetKind, ShrapnelBulletType, ShrapnelDrawPlan, ShrapnelTrianglePlan,
    SpaceLiquidBulletType,
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
pub use effect::{
    shake_intensity, Effect, EffectContainer, EffectCreateContext, EffectCreatePlan, EffectInterp,
    EffectParent, EffectRegistry, EffectRenderParams, EffectSpawnPlan, ExplosionDrawPlan,
    ExplosionEffect, ExplosionSmokePlan, ExplosionSparkPlan, ExplosionWavePlan, MultiEffect,
    ParticleDrawItem, ParticleDrawKind, ParticleDrawPlan, ParticleEffect, ParticleVectorInput,
    RadialEffect, SeqEffect, SeqRenderPlan, SoundEffect, SoundEffectCreatePlan, SoundPlaybackPlan,
    WaveDrawPlan, WaveEffect, WrapEffect, DEFAULT_EFFECT_CLIP, DEFAULT_EFFECT_LAYER,
    DEFAULT_EFFECT_LIFETIME, SHAKE_FALLOFF,
};
pub use entity_collisions::{
    collide, legs_solid, move_check_hitbox, move_delta_rect, move_hitbox, move_rect, overlaps_tile,
    solid, water_solid, CollisionMoveResult, CollisionPoint, MAX_DELTA, SEGMENT,
};
pub use entity_group::{EntityGroup, EntityGroupItem, Rect, SpatialEntity};
pub use entity_indexer::EntityIndexer;
pub use fires::{ExtinguishResult, FireCreateResult, FireRules, Fires, BASE_FIRE_LIFETIME};
pub use group_defs::{
    colliding_groups, group_def, mapping_groups, spatial_groups, EntityGroupKind, GroupDef,
    GROUP_DEFS,
};
pub use leg::Leg;
pub use leg_destroy_data::{LegDestroyData, TextureRegionRef};
pub use lightning::{
    create_lightning_plan, find_furthest_target, within_lightning_rect, LightningConfig,
    LightningHitter, LightningInsulatorHit, LightningPlan, LightningPoint, LightningSeedState,
    LightningSpawnPlan, LightningTarget, HIT_RANGE, MAX_CHAIN,
};
pub use mover::Mover;
pub use part::{
    DrawPartConfig, EffectSpawnerDrawPlan, EffectSpawnerPart, EffectSpawnerRectPlan,
    EffectSpawnerSpawnPlan, FlarePart, FlarePartDrawPlan, FlareTrianglePlan, HaloPart,
    HaloPartDrawPlan, HaloShapeKind, HaloShapePlan, HoverCirclePlan, HoverPart, HoverPartDrawPlan,
    PartMove, PartParams, PartProgress, RegionDrawItem, RegionDrawKind, RegionPart,
    RegionPartDrawPlan, RegionPartLoadPlan, RegionTexture, ShapePart, ShapePartDrawItem,
    ShapePartDrawPlan, ShapePartKind,
};
pub use pattern::{
    BulletHandler, ShootAlternate, ShootBarrel, ShootHelix, ShootMulti, ShootPattern, ShootSine,
    ShootSpread, ShootSummon, Shot,
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
pub use units::{
    units_any, units_any_entities, units_any_entities_centered, units_best_enemy,
    units_best_target, units_can_create, units_can_interact, units_closest, units_closest_building,
    units_closest_enemy, units_closest_in_range, units_closest_overlap, units_closest_target,
    units_count, units_find_enemy_tile, units_get_cap, units_get_string_cap,
    units_invalidate_target, units_is_hittable, units_near_enemy, units_unit_cap_death_plan,
    units_unit_death_plan, units_unit_despawn_plan, units_unit_destroy_plan,
    units_unit_env_death_plan, units_unit_safe_death_plan, AiBlockStatus, AiCircleAttackInput,
    AiCircleInput, AiCircleTarget, AiController, AiControllerTimers, AiFaceMovementInput,
    AiFacePlan, AiFaceTargetInput, AiFlaggedTarget, AiMountInput, AiMountPlan, AiMovePlan,
    AiMoveToInput, AiPathfindInput, AiTargetSnapshot, AiUnloadPayloadInput, AiVisualInput,
    AiVisualPlan, AiWeaponInfo, AiWeaponPlan, AiWeaponUpdateInput, BuildPlan, StatusEntry,
    UnitCapRules, UnitCapTeam, UnitCapType, UnitController, UnitLifecycleEffect, UnitLifecyclePlan,
    UnitLifecycleSnapshot, UnitsEntityTileSnapshot, UnitsRect, UnitsTargetKind,
    UnitsTargetSnapshot, UnitsTeamPresence, WeaponMount, AI_ROTATE_BACK_TIMER, AI_TIMER_COUNT,
    AI_TIMER_TARGET, AI_TIMER_TARGET2, AI_TIMER_TARGET3, AI_TIMER_TARGET4, UNITS_CAP_INFINITY,
};

// Mirrors upstream core/src/mindustry/entities. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.
