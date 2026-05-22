// Mirrors upstream core/src/mindustry/service. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod achievement;
pub mod game_service;
pub mod s_stat;

pub use achievement::{
    Achievement, AchievementContext, AchievementData, AchievementService, AchievementState,
};
pub use game_service::{
    DefaultGameService, GameService, GameServiceBlockBuildPlan, GameServiceBlockBuildSnapshot,
    GameServiceBlockDestroySnapshot, GameServiceBuildingBulletDestroySnapshot,
    GameServiceClientChatSnapshot, GameServiceClientPreConnectSnapshot, GameServiceEventPlan,
    GameServiceFrameUpdateSnapshot, GameServiceGeneratorPressureExplodeSnapshot,
    GameServiceInitAction, GameServiceLaunchItemSnapshot, GameServiceNewGameSnapshot,
    GameServiceNucleusGroundZeroSnapshot, GameServicePayloadDropSnapshot,
    GameServicePickupSnapshot, GameServicePlayerJoinSnapshot, GameServiceRegisterSnapshot,
    GameServiceResearchSnapshot, GameServiceSaveLoadSnapshot, GameServiceSectorCaptureSnapshot,
    GameServiceSectorLaunchLoadoutSnapshot, GameServiceState, GameServiceTriggerSnapshot,
    GameServiceTurnPlan, GameServiceTurnSnapshot, GameServiceUnitBulletDestroySnapshot,
    GameServiceUnitControlSnapshot, GameServiceUnitCreatePlan, GameServiceUnitCreateSnapshot,
    GameServiceUnitDestroySnapshot, GameServiceUnitDrownSnapshot, GameServiceUnlockSnapshot,
    GameServiceUpdatePlan, GameServiceUpdateSnapshot, GameServiceWaveSnapshot,
    GameServiceWinSnapshot,
};
pub use s_stat::{SStat, StatService};
