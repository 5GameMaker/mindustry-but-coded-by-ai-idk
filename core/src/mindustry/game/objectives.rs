//! Campaign/tech-tree objectives mirroring upstream `mindustry.game.Objectives`.
//!
//! The Java class mixes completion checks with UI bundle formatting. This Rust
//! port keeps the completion state and returns stable display tokens that a
//! future UI/localization layer can render.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveContent {
    pub name: String,
    pub localized_name: String,
    pub emoji: String,
    pub unlocked_host: bool,
    pub parent_unlocked_host: bool,
}

impl ObjectiveContent {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            name,
            emoji: String::new(),
            unlocked_host: false,
            parent_unlocked_host: true,
        }
    }

    pub fn localized(mut self, localized_name: impl Into<String>) -> Self {
        self.localized_name = localized_name.into();
        self
    }

    pub fn emoji(mut self, emoji: impl Into<String>) -> Self {
        self.emoji = emoji.into();
        self
    }

    pub fn unlocked_host(mut self, unlocked: bool) -> Self {
        self.unlocked_host = unlocked;
        self
    }

    pub fn parent_unlocked_host(mut self, unlocked: bool) -> Self {
        self.parent_unlocked_host = unlocked;
        self
    }

    pub fn icon_text(&self) -> String {
        if self.emoji.is_empty() {
            self.localized_name.clone()
        } else {
            format!("{} {}", self.emoji, self.localized_name)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorObjectiveState {
    pub name: String,
    pub localized_name: String,
    pub save_exists: bool,
    pub captured: bool,
    pub has_base: bool,
}

impl SectorObjectiveState {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            name,
            save_exists: false,
            captured: false,
            has_base: false,
        }
    }

    pub fn localized(mut self, localized_name: impl Into<String>) -> Self {
        self.localized_name = localized_name.into();
        self
    }

    pub fn save_exists(mut self, save_exists: bool) -> Self {
        self.save_exists = save_exists;
        self
    }

    pub fn captured(mut self, captured: bool) -> Self {
        self.captured = captured;
        self
    }

    pub fn has_base(mut self, has_base: bool) -> Self {
        self.has_base = has_base;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanetObjectiveState {
    pub name: String,
    pub localized_name: String,
    pub sectors_have_base: Vec<bool>,
}

impl PlanetObjectiveState {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            localized_name: name.clone(),
            name,
            sectors_have_base: Vec::new(),
        }
    }

    pub fn localized(mut self, localized_name: impl Into<String>) -> Self {
        self.localized_name = localized_name.into();
        self
    }

    pub fn sectors_have_base(mut self, sectors_have_base: impl IntoIterator<Item = bool>) -> Self {
        self.sectors_have_base = sectors_have_base.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectiveKind {
    Research(ObjectiveContent),
    Produce(ObjectiveContent),
    SectorComplete(SectorObjectiveState),
    OnSector(SectorObjectiveState),
    OnPlanet(PlanetObjectiveState),
}

impl ObjectiveKind {
    pub fn complete(&self) -> bool {
        match self {
            ObjectiveKind::Research(content) | ObjectiveKind::Produce(content) => {
                content.unlocked_host
            }
            ObjectiveKind::SectorComplete(sector) => {
                sector.save_exists && sector.captured && sector.has_base
            }
            ObjectiveKind::OnSector(sector) => sector.has_base,
            ObjectiveKind::OnPlanet(planet) => planet.sectors_have_base.iter().any(|&value| value),
        }
    }

    pub fn display_token(&self) -> String {
        match self {
            ObjectiveKind::Research(content) => format!(
                "requirement.research:{}",
                if content.parent_unlocked_host {
                    content.icon_text()
                } else {
                    "???".to_string()
                }
            ),
            ObjectiveKind::Produce(content) => format!(
                "requirement.produce:{}",
                if content.unlocked_host {
                    content.icon_text()
                } else {
                    "???".to_string()
                }
            ),
            ObjectiveKind::SectorComplete(sector) => {
                format!("requirement.capture:{}", sector.localized_name)
            }
            ObjectiveKind::OnSector(sector) => {
                format!("requirement.onsector:{}", sector.localized_name)
            }
            ObjectiveKind::OnPlanet(planet) => {
                format!("requirement.onplanet:{}", planet.localized_name)
            }
        }
    }

    pub fn java_to_string(&self) -> String {
        match self {
            ObjectiveKind::Research(content) => format!("research: {}", content.name),
            ObjectiveKind::Produce(content) => format!("produce: {}", content.name),
            ObjectiveKind::SectorComplete(sector) => {
                format!("sectorComplete: {}", sector.name)
            }
            ObjectiveKind::OnSector(sector) => format!("onSector: {}", sector.name),
            ObjectiveKind::OnPlanet(planet) => format!("onPlanet: {}", planet.name),
        }
    }
}

pub trait Objective {
    fn complete(&self) -> bool;
    fn display_token(&self) -> String;
}

impl Objective for ObjectiveKind {
    fn complete(&self) -> bool {
        ObjectiveKind::complete(self)
    }

    fn display_token(&self) -> String {
        ObjectiveKind::display_token(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_objective_matches_java_parent_visibility_and_unlock_check() {
        let hidden_parent = ObjectiveKind::Research(
            ObjectiveContent::new("thorium")
                .localized("Thorium")
                .emoji("[thorium]")
                .parent_unlocked_host(false),
        );
        assert!(!hidden_parent.complete());
        assert_eq!(hidden_parent.display_token(), "requirement.research:???");
        assert_eq!(hidden_parent.java_to_string(), "research: thorium");

        let visible = ObjectiveKind::Research(
            ObjectiveContent::new("copper")
                .localized("Copper")
                .emoji("[copper]")
                .unlocked_host(true),
        );
        assert!(visible.complete());
        assert_eq!(
            visible.display_token(),
            "requirement.research:[copper] Copper"
        );
    }

    #[test]
    fn produce_objective_hides_content_until_unlocked_like_java() {
        let locked = ObjectiveKind::Produce(ObjectiveContent::new("dagger").localized("Dagger"));
        assert!(!locked.complete());
        assert_eq!(locked.display_token(), "requirement.produce:???");

        let unlocked = ObjectiveKind::Produce(
            ObjectiveContent::new("dagger")
                .localized("Dagger")
                .emoji("[dagger]")
                .unlocked_host(true),
        );
        assert!(unlocked.complete());
        assert_eq!(
            unlocked.display_token(),
            "requirement.produce:[dagger] Dagger"
        );
        assert_eq!(unlocked.java_to_string(), "produce: dagger");
    }

    #[test]
    fn sector_and_planet_objectives_match_java_completion_conditions() {
        let sector = SectorObjectiveState::new("groundZero")
            .localized("Ground Zero")
            .save_exists(true)
            .captured(true)
            .has_base(false);
        let complete = ObjectiveKind::SectorComplete(sector.clone());
        assert!(!complete.complete());
        assert_eq!(complete.display_token(), "requirement.capture:Ground Zero");
        assert_eq!(complete.java_to_string(), "sectorComplete: groundZero");

        let on_sector = ObjectiveKind::OnSector(sector.clone().has_base(true));
        assert!(on_sector.complete());
        assert_eq!(
            on_sector.display_token(),
            "requirement.onsector:Ground Zero"
        );

        let planet = ObjectiveKind::OnPlanet(
            PlanetObjectiveState::new("serpulo")
                .localized("Serpulo")
                .sectors_have_base([false, true, false]),
        );
        assert!(planet.complete());
        assert_eq!(planet.display_token(), "requirement.onplanet:Serpulo");
        assert_eq!(planet.java_to_string(), "onPlanet: serpulo");
    }
}
