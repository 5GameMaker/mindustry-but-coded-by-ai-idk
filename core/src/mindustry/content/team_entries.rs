use crate::mindustry::r#type::TeamEntry;

pub fn load() -> Vec<TeamEntry> {
    // Upstream `TeamEntries.load()` currently contains only TODO comments and
    // does not instantiate vanilla TeamEntry content. Keep this intentionally
    // empty so ContentType::Team is absent from vanilla content headers.
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_team_entries_match_upstream_empty_loader() {
        let entries = load();
        assert!(entries.is_empty());
    }
}
