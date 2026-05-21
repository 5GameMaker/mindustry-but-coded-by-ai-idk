pub mod abilities;
pub mod comp;
pub mod entity_indexer;
pub mod leg;
pub mod leg_destroy_data;
pub mod mover;
pub mod pattern;
pub mod sized;
pub mod unit_sorts;
pub mod units;

pub use abilities::{
    Ability, BasicAbility, LiquidExplodeAbility, LiquidRegenAbility, RegenAbility,
    SpawnDeathAbility,
};
pub use comp::{apply_armor, ShieldComp};
pub use comp::{BuildingTetherAction, BuildingTetherComp, BuildingTetherRef};
pub use comp::{
    ChildComp, ChildParent, DamageComp, DamageState, DrawComp, ElevationMoveComp, Interval,
    OwnerComp, PosComp, PosTeamDef, RotComp, ShielderComp, SolidPred, TeamState, TimerComp,
};
pub use comp::{EntityComp, EntityIoState, EntityLocality};
pub use comp::{HealthComp, ItemStackSlot, ItemsComp};
pub use comp::{HitboxComp, HitboxRect};
pub use comp::{PowerGraphUpdate, PowerGraphUpdaterComp};
pub use comp::{SyncComp, SyncHooks};
pub use comp::{TeamComp, TeamRulesView, VelComp};
pub use comp::{TimedComp, TimedKillComp};
pub use comp::{UnitTetherAction, UnitTetherComp, UnitTetherRef};
pub use comp::{WorldLabelAlign, WorldLabelComp, WorldLabelDrawPlan};
pub use entity_indexer::EntityIndexer;
pub use leg::Leg;
pub use leg_destroy_data::{LegDestroyData, TextureRegionRef};
pub use mover::Mover;
pub use pattern::{
    BulletHandler, ShootAlternate, ShootBarrel, ShootHelix, ShootPattern, ShootSine, ShootSpread,
    ShootSummon, Shot,
};
pub use sized::{EntityPosition, SizedEntity};
pub use unit_sorts::{
    building_default, building_water, closest, farthest, strongest, weakest,
    BuildingPriorityTarget, SortTarget,
};
pub use units::{BuildPlan, StatusEntry, WeaponMount};

// Mirrors upstream core/src/mindustry/entities. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.
