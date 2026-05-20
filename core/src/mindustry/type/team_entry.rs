use crate::mindustry::ctype::{Content, ContentId, ContentType, UnlockableContentBase};

#[derive(Debug, Clone, PartialEq)]
pub struct TeamEntry {
    pub base: UnlockableContentBase,
    pub team_id: i32,
}

impl TeamEntry {
    pub fn new(id: ContentId, name: impl Into<String>, team_id: i32) -> Self {
        Self {
            base: UnlockableContentBase::new(id, ContentType::Team, name),
            team_id,
        }
    }

    pub fn from_team_name(id: ContentId, name: impl Into<String>, team_id: i32) -> Self {
        Self::new(id, name, team_id)
    }

    pub fn name(&self) -> &str {
        &self.base.mappable.name
    }

    pub fn lore_bundle_key(&self) -> String {
        format!("@team.{}.log", self.name())
    }
}

impl Content for TeamEntry {
    fn id(&self) -> ContentId {
        self.base.mappable.base.id
    }

    fn content_type(&self) -> ContentType {
        ContentType::Team
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_entry_matches_java_content_type_and_lore_key() {
        let entry = TeamEntry::new(3, "crux", 2);
        assert_eq!(entry.id(), 3);
        assert_eq!(entry.content_type(), ContentType::Team);
        assert_eq!(entry.name(), "crux");
        assert_eq!(entry.team_id, 2);
        assert_eq!(entry.lore_bundle_key(), "@team.crux.log");
        assert_eq!(TeamEntry::from_team_name(4, "sharded", 1).name(), "sharded");
    }
}
