// Mirrors upstream core/src/mindustry/audio. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod sound_control;
pub mod sound_loop;
pub mod sound_priority;

pub use sound_control::{
    FilterFadePlan, LoopSoundAction, LoopSoundPlan, MusicAction, MusicPlayInput, MusicPlaybackPlan,
    MusicRegisterPlan, SoundBusPlan, SoundControl, SoundDarkChance, SoundData,
};
pub use sound_loop::{SoundLoop, SoundLoopBackend};
pub use sound_priority::{
    SoundAssetLength, SoundFloatSetting, SoundGroupSetting, SoundIntSetting, SoundPriority,
    SoundPriorityPlan,
};
