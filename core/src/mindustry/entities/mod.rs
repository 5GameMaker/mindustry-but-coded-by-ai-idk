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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityClassIdEntry {
    pub name: &'static str,
    pub id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityClassKind {
    Player,
    Unit,
    Bullet,
    Decal,
    Effect,
    Fire,
    Puddle,
    Weather,
    Other,
}

pub const PLAYER_CLASS_ID: u8 = 12;
pub const BULLET_CLASS_ID: u8 = 7;
pub const DECAL_CLASS_ID: u8 = 8;
pub const EFFECT_STATE_CLASS_ID: u8 = 9;
pub const FIRE_CLASS_ID: u8 = 10;
pub const PUDDLE_CLASS_ID: u8 = 13;
pub const WEATHER_STATE_CLASS_ID: u8 = 14;

/// Java `annotations/src/main/resources/classids.properties` migrated into a
/// Rust lookup table. This is the stable outer `entity.classId()` byte used by
/// `NetServer.entitySnapshot(...)` and `NetClient.readSyncEntity(...)`.
pub const ENTITY_CLASS_IDS: &[EntityClassIdEntry] = &[
    EntityClassIdEntry {
        name: "alpha",
        id: 0,
    },
    EntityClassIdEntry {
        name: "arkyid",
        id: 29,
    },
    EntityClassIdEntry {
        name: "atrax",
        id: 1,
    },
    EntityClassIdEntry {
        name: "beta",
        id: 30,
    },
    EntityClassIdEntry {
        name: "block",
        id: 2,
    },
    EntityClassIdEntry {
        name: "corvus",
        id: 24,
    },
    EntityClassIdEntry {
        name: "elude",
        id: 45,
    },
    EntityClassIdEntry {
        name: "flare",
        id: 3,
    },
    EntityClassIdEntry {
        name: "gamma",
        id: 31,
    },
    EntityClassIdEntry {
        name: "latum",
        id: 46,
    },
    EntityClassIdEntry {
        name: "mace",
        id: 4,
    },
    EntityClassIdEntry {
        name: "manifold",
        id: 36,
    },
    EntityClassIdEntry {
        name: "mega",
        id: 5,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.BuildingComp",
        id: 6,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.BulletComp",
        id: BULLET_CLASS_ID,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.DecalComp",
        id: DECAL_CLASS_ID,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.EffectStateComp",
        id: EFFECT_STATE_CLASS_ID,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.FireComp",
        id: 10,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.LaunchCoreComp",
        id: 11,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.LocationPingComp",
        id: 48,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.PlayerComp",
        id: PLAYER_CLASS_ID,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.PosTeam",
        id: 27,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.PosTeamDef",
        id: 28,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.PowerGraphComp",
        id: 41,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.PowerGraphUpdaterComp",
        id: 42,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.PuddleComp",
        id: PUDDLE_CLASS_ID,
    },
    EntityClassIdEntry {
        name: "mindustry.entities.comp.WorldLabelComp",
        id: 35,
    },
    EntityClassIdEntry {
        name: "mindustry.type.Weather.WeatherStateComp",
        id: WEATHER_STATE_CLASS_ID,
    },
    EntityClassIdEntry {
        name: "mindustry.world.blocks.campaign.LaunchPad.LaunchPayloadComp",
        id: 15,
    },
    EntityClassIdEntry {
        name: "mindustry.world.blocks.campaign.PayloadLaunchPad.LargeLaunchPayloadComp",
        id: 34,
    },
    EntityClassIdEntry {
        name: "mindustry.world.blocks.defense.ForceProjector.ForceDrawComp",
        id: 22,
    },
    EntityClassIdEntry {
        name: "missile",
        id: 39,
    },
    EntityClassIdEntry {
        name: "mono",
        id: 16,
    },
    EntityClassIdEntry {
        name: "nova",
        id: 17,
    },
    EntityClassIdEntry {
        name: "oct",
        id: 26,
    },
    EntityClassIdEntry {
        name: "osc",
        id: 44,
    },
    EntityClassIdEntry {
        name: "poly",
        id: 18,
    },
    EntityClassIdEntry {
        name: "pulsar",
        id: 19,
    },
    EntityClassIdEntry {
        name: "quad",
        id: 23,
    },
    EntityClassIdEntry {
        name: "quasar",
        id: 32,
    },
    EntityClassIdEntry {
        name: "renale",
        id: 47,
    },
    EntityClassIdEntry {
        name: "risso",
        id: 20,
    },
    EntityClassIdEntry {
        name: "spiroct",
        id: 21,
    },
    EntityClassIdEntry {
        name: "stell",
        id: 43,
    },
    EntityClassIdEntry {
        name: "timed",
        id: 38,
    },
    EntityClassIdEntry {
        name: "timedDef",
        id: 37,
    },
    EntityClassIdEntry {
        name: "toxopid",
        id: 33,
    },
    EntityClassIdEntry {
        name: "vanquish",
        id: 40,
    },
    EntityClassIdEntry {
        name: "vela",
        id: 25,
    },
];

pub fn entity_class_id(name: &str) -> Option<u8> {
    ENTITY_CLASS_IDS
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.id)
}

pub fn entity_class_name(id: u8) -> Option<&'static str> {
    ENTITY_CLASS_IDS
        .iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.name)
}

pub fn entity_class_kind(id: u8) -> Option<EntityClassKind> {
    let name = entity_class_name(id)?;
    if id == PLAYER_CLASS_ID {
        Some(EntityClassKind::Player)
    } else if id == BULLET_CLASS_ID {
        Some(EntityClassKind::Bullet)
    } else if id == DECAL_CLASS_ID {
        Some(EntityClassKind::Decal)
    } else if id == EFFECT_STATE_CLASS_ID {
        Some(EntityClassKind::Effect)
    } else if id == FIRE_CLASS_ID {
        Some(EntityClassKind::Fire)
    } else if id == PUDDLE_CLASS_ID {
        Some(EntityClassKind::Puddle)
    } else if id == WEATHER_STATE_CLASS_ID {
        Some(EntityClassKind::Weather)
    } else if name.contains('.') {
        Some(EntityClassKind::Other)
    } else {
        Some(EntityClassKind::Unit)
    }
}

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
pub use comp::{PlayerComp, PlayerUnitState, PlayerUnitSwitchContext};
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn entity_class_ids_match_upstream_classids_properties_baseline() {
        assert_eq!(ENTITY_CLASS_IDS.len(), 49);
        assert_eq!(
            entity_class_id("mindustry.entities.comp.PlayerComp"),
            Some(PLAYER_CLASS_ID)
        );
        assert_eq!(
            entity_class_name(PLAYER_CLASS_ID),
            Some("mindustry.entities.comp.PlayerComp")
        );
        assert_eq!(entity_class_id("alpha"), Some(0));
        assert_eq!(entity_class_id("block"), Some(2));
        assert_eq!(
            entity_class_id("mindustry.entities.comp.BulletComp"),
            Some(BULLET_CLASS_ID)
        );
        assert_eq!(
            entity_class_id("mindustry.type.Weather.WeatherStateComp"),
            Some(14)
        );
        assert_eq!(
            entity_class_id("mindustry.entities.comp.LocationPingComp"),
            Some(48)
        );
        assert_eq!(
            entity_class_kind(PLAYER_CLASS_ID),
            Some(EntityClassKind::Player)
        );
        assert_eq!(
            entity_class_id("mindustry.entities.comp.DecalComp"),
            Some(DECAL_CLASS_ID)
        );
        assert_eq!(
            entity_class_kind(DECAL_CLASS_ID),
            Some(EntityClassKind::Decal)
        );
        assert_eq!(
            entity_class_id("mindustry.entities.comp.EffectStateComp"),
            Some(EFFECT_STATE_CLASS_ID)
        );
        assert_eq!(
            entity_class_kind(EFFECT_STATE_CLASS_ID),
            Some(EntityClassKind::Effect)
        );
        assert_eq!(entity_class_kind(2), Some(EntityClassKind::Unit));
        assert_eq!(
            entity_class_kind(FIRE_CLASS_ID),
            Some(EntityClassKind::Fire)
        );
        assert_eq!(
            entity_class_id("mindustry.entities.comp.PuddleComp"),
            Some(PUDDLE_CLASS_ID)
        );
        assert_eq!(
            entity_class_kind(PUDDLE_CLASS_ID),
            Some(EntityClassKind::Puddle)
        );
        assert_eq!(
            entity_class_id("mindustry.type.Weather.WeatherStateComp"),
            Some(WEATHER_STATE_CLASS_ID)
        );
        assert_eq!(
            entity_class_kind(WEATHER_STATE_CLASS_ID),
            Some(EntityClassKind::Weather)
        );
        assert_eq!(
            entity_class_kind(BULLET_CLASS_ID),
            Some(EntityClassKind::Bullet)
        );
        assert_eq!(entity_class_kind(255), None);

        let mut names = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for entry in ENTITY_CLASS_IDS {
            assert!(
                names.insert(entry.name),
                "duplicate entity class name {}",
                entry.name
            );
            assert!(
                ids.insert(entry.id),
                "duplicate entity class id {}",
                entry.id
            );
        }
    }
}

// Mirrors upstream core/src/mindustry/entities. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.
