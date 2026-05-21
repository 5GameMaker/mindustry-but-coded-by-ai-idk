//! Incremental Rust mirror of upstream `mindustry.entities.comp`.

pub mod damage;
pub mod owner;
pub mod rot;

pub use damage::DamageComp;
pub use owner::OwnerComp;
pub use rot::RotComp;
