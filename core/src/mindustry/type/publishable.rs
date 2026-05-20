use std::path::PathBuf;

/// Defines a piece of content that can be published on the Workshop.
pub trait Publishable {
    /// @return workshop item ID, or None if this isn't on the workshop.
    fn get_steam_id(&self) -> Option<String>;

    /// Adds a steam ID to this item once it's published.
    fn add_steam_id(&mut self, id: String);

    /// Removes the item ID; called when the item isn't found.
    fn remove_steam_id(&mut self);

    /// @return default title of the listing.
    fn steam_title(&self) -> String;

    /// @return standard steam listing description, may be None.
    fn steam_description(&self) -> Option<String>;

    /// @return the tag that this content has. e.g. 'schematic' or 'map'.
    fn steam_tag(&self) -> String;

    /// @return a folder with everything needed for this piece of content in it.
    fn create_steam_folder(&self, id: &str) -> PathBuf;

    /// @return a preview file PNG.
    fn create_steam_preview(&self, id: &str) -> PathBuf;

    /// @return any extra tags to add to this item.
    fn extra_tags(&self) -> Vec<String> {
        Vec::new()
    }

    /// Mirrors `Vars.steam` for callers that have platform state available.
    ///
    /// Java only reports an item as having a Steam ID when both the stored ID is
    /// present and Steam integration is enabled. Headless/offline tests can
    /// override this instead of reaching for a global.
    fn steam_enabled(&self) -> bool {
        true
    }

    /// @return whether this item is or was once on the workshop.
    fn has_steam_id(&self) -> bool {
        self.get_steam_id().is_some() && self.steam_enabled()
    }

    /// Called before this item is published.
    fn pre_publish(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct DummyPublishable {
        steam_id: Option<String>,
        steam_enabled: bool,
    }

    impl Publishable for DummyPublishable {
        fn get_steam_id(&self) -> Option<String> {
            self.steam_id.clone()
        }

        fn add_steam_id(&mut self, id: String) {
            self.steam_id = Some(id);
        }

        fn remove_steam_id(&mut self) {
            self.steam_id = None;
        }

        fn steam_title(&self) -> String {
            "title".into()
        }

        fn steam_description(&self) -> Option<String> {
            None
        }

        fn steam_tag(&self) -> String {
            "map".into()
        }

        fn create_steam_folder(&self, id: &str) -> PathBuf {
            PathBuf::from(format!("folder-{id}"))
        }

        fn create_steam_preview(&self, id: &str) -> PathBuf {
            PathBuf::from(format!("preview-{id}.png"))
        }

        fn steam_enabled(&self) -> bool {
            self.steam_enabled
        }
    }

    #[test]
    fn publishable_defaults_follow_java_interface_methods() {
        let mut item = DummyPublishable::default();
        assert!(item.extra_tags().is_empty());
        assert!(item.pre_publish());
        assert!(!item.has_steam_id());

        item.add_steam_id("12345".into());
        assert!(!item.has_steam_id());
        item.steam_enabled = true;
        assert!(item.has_steam_id());

        item.remove_steam_id();
        assert!(!item.has_steam_id());
        assert_eq!(item.steam_title(), "title");
        assert_eq!(item.steam_description(), None);
        assert_eq!(item.steam_tag(), "map");
        assert_eq!(item.create_steam_folder("42"), PathBuf::from("folder-42"));
        assert_eq!(
            item.create_steam_preview("42"),
            PathBuf::from("preview-42.png")
        );
    }
}
