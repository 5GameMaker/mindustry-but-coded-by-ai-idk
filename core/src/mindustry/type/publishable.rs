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

    /// @return whether this item is or was once on the workshop.
    fn has_steam_id(&self) -> bool {
        self.get_steam_id().is_some()
    }

    /// Called before this item is published.
    fn pre_publish(&self) -> bool {
        true
    }
}
