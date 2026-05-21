//! Dummy target definition mirroring upstream
//! `mindustry.entities.comp.PosTeamDef`.
//!
//! Java only carries an `@EntityDef(value = Teamc.class, genio = false,
//! isFinal = false)` annotation here. Rust keeps that metadata as constants so
//! the future entity-definition/codegen layer can consume it explicitly.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PosTeamDef;

impl PosTeamDef {
    pub const ENTITY_COMPONENT: &'static str = "Teamc";
    pub const GENIO: bool = false;
    pub const IS_FINAL: bool = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_team_def_preserves_java_entity_definition_metadata() {
        assert_eq!(PosTeamDef::ENTITY_COMPONENT, "Teamc");
        assert!(!PosTeamDef::GENIO);
        assert!(!PosTeamDef::IS_FINAL);
        assert_eq!(PosTeamDef, PosTeamDef::default());
    }
}
