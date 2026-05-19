use crate::mindustry::ctype::ContentType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamEntry {
    pub name: String,
    pub team_id: i32,
}

impl TeamEntry {
    pub fn new(name: impl Into<String>, team_id: i32) -> Self {
        Self {
            name: name.into(),
            team_id,
        }
    }

    pub fn from_team_name(name: impl Into<String>, team_id: i32) -> Self {
        Self::new(name, team_id)
    }

    pub const fn content_type(&self) -> ContentType {
        ContentType::Team
    }

    pub fn lore_bundle_key(&self) -> String {
        format!("@team.{}.log", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_entry_matches_java_content_type_and_lore_key() {
        let entry = TeamEntry::new("crux", 1);
        assert_eq!(entry.content_type(), ContentType::Team);
        assert_eq!(entry.lore_bundle_key(), "@team.crux.log");
        assert_eq!(TeamEntry::from_team_name("sharded", 0).name, "sharded");
    }
}
