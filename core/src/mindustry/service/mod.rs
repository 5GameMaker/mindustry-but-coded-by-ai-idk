// Mirrors upstream core/src/mindustry/service. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod achievement;
pub mod game_service;
pub mod s_stat;

pub use achievement::{
    Achievement, AchievementContext, AchievementData, AchievementService, AchievementState,
};
pub use game_service::{
    DefaultGameService, GameService, GameServiceInitAction, GameServiceState,
    GameServiceUpdatePlan, GameServiceUpdateSnapshot,
};
pub use s_stat::{SStat, StatService};
