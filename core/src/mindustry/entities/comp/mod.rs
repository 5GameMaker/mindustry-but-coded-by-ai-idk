//! Incremental Rust mirror of upstream `mindustry.entities.comp`.

pub mod damage;
pub mod draw;
pub mod owner;
pub mod pos_team_def;
pub mod rot;

pub use damage::DamageComp;
pub use draw::DrawComp;
pub use owner::OwnerComp;
pub use pos_team_def::PosTeamDef;
pub use rot::RotComp;
