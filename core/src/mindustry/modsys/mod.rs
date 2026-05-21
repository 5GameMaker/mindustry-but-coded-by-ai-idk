//! Incremental Rust mirror of upstream `mindustry.mod`.
//!
//! `mod` is a Rust keyword, so this crate exposes the package as `modsys`.
//! The first migrated pieces are the Java `Mod` base class hooks and the
//! `NoPatch` marker annotation.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModConfigPaths {
    pub mods_dir: String,
    pub plugin_name: String,
}

impl ModConfigPaths {
    pub fn new(mods_dir: impl Into<String>, plugin_name: impl Into<String>) -> Self {
        Self {
            mods_dir: trim_slash(mods_dir.into()),
            plugin_name: plugin_name.into(),
        }
    }

    /// Java: `mods/[plugin-name]`.
    pub fn config_folder(&self) -> String {
        format!("{}/{}", self.mods_dir, self.plugin_name)
    }

    /// Java: `mods/[plugin-name]/config.json`.
    pub fn config(&self) -> String {
        format!("{}/config.json", self.config_folder())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: String,
    pub description: String,
}

impl CommandSpec {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandRegistry {
    commands: Vec<CommandSpec>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, name: impl Into<String>, description: impl Into<String>) {
        self.commands.push(CommandSpec::new(name, description));
    }

    pub fn commands(&self) -> &[CommandSpec] {
        &self.commands
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpritePacker {
    textures: Vec<String>,
}

impl SpritePacker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_texture(&mut self, path: impl Into<String>) {
        self.textures.push(path.into());
    }

    pub fn textures(&self) -> &[String] {
        &self.textures
    }
}

/// Rust equivalent of the Java abstract `Mod` base class.
///
/// Every hook defaults to a no-op, matching upstream.
pub trait Mod {
    fn get_config_folder(&self, paths: &ModConfigPaths) -> String {
        paths.config_folder()
    }

    fn get_config(&self, paths: &ModConfigPaths) -> String {
        paths.config()
    }

    /// Called after all plugins have been created and commands registered.
    fn init(&mut self) {}

    /// Called on clientside mods. Load content here.
    fn load_content(&mut self) {}

    /// Called during sprite packing to allow adding custom textures.
    fn pack_sprites(&mut self, _packer: &mut SpritePacker) {}

    /// Register commands to be used on the server side.
    fn register_server_commands(&mut self, _handler: &mut CommandRegistry) {}

    /// Register commands to be used on the client side.
    fn register_client_commands(&mut self, _handler: &mut CommandRegistry) {}
}

/// Marker equivalent to Java's runtime `@NoPatch` annotation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NoPatch;

pub trait NoPatchMarker {}

fn trim_slash(mut path: String) -> String {
    while path.ends_with('/') || path.ends_with('\\') {
        path.pop();
    }
    if path.is_empty() {
        ".".into()
    } else {
        path.replace('\\', "/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct EmptyMod;

    impl Mod for EmptyMod {}

    #[derive(Default)]
    struct RecordingMod {
        initialized: bool,
        content_loaded: bool,
    }

    impl Mod for RecordingMod {
        fn init(&mut self) {
            self.initialized = true;
        }

        fn load_content(&mut self) {
            self.content_loaded = true;
        }

        fn pack_sprites(&mut self, packer: &mut SpritePacker) {
            packer.add_texture("sprites/custom.png");
        }

        fn register_server_commands(&mut self, handler: &mut CommandRegistry) {
            handler.register("status", "prints status");
        }

        fn register_client_commands(&mut self, handler: &mut CommandRegistry) {
            handler.register("ping", "client ping");
        }
    }

    struct LockedField;
    impl NoPatchMarker for LockedField {}

    fn assert_no_patch<T: NoPatchMarker>() {}

    #[test]
    fn mod_config_paths_follow_java_mods_plugin_layout() {
        let paths = ModConfigPaths::new("mods\\", "example");

        assert_eq!(paths.config_folder(), "mods/example");
        assert_eq!(paths.config(), "mods/example/config.json");

        let empty_root = ModConfigPaths::new("", "plugin");
        assert_eq!(empty_root.config_folder(), "./plugin");
    }

    #[test]
    fn default_mod_hooks_are_noops_like_java_base_class() {
        let paths = ModConfigPaths::new("mods", "empty");
        let mut module = EmptyMod;
        let mut packer = SpritePacker::new();
        let mut server = CommandRegistry::new();
        let mut client = CommandRegistry::new();

        assert_eq!(module.get_config_folder(&paths), "mods/empty");
        assert_eq!(module.get_config(&paths), "mods/empty/config.json");

        module.init();
        module.load_content();
        module.pack_sprites(&mut packer);
        module.register_server_commands(&mut server);
        module.register_client_commands(&mut client);

        assert!(packer.textures().is_empty());
        assert!(server.is_empty());
        assert!(client.is_empty());
    }

    #[test]
    fn mod_hooks_can_register_content_sprites_and_commands() {
        let mut module = RecordingMod::default();
        let mut packer = SpritePacker::new();
        let mut server = CommandRegistry::new();
        let mut client = CommandRegistry::new();

        module.init();
        module.load_content();
        module.pack_sprites(&mut packer);
        module.register_server_commands(&mut server);
        module.register_client_commands(&mut client);

        assert!(module.initialized);
        assert!(module.content_loaded);
        assert_eq!(packer.textures(), &["sprites/custom.png".to_string()]);
        assert_eq!(
            server.commands(),
            &[CommandSpec::new("status", "prints status")]
        );
        assert_eq!(
            client.commands(),
            &[CommandSpec::new("ping", "client ping")]
        );
    }

    #[test]
    fn no_patch_marker_can_be_attached_to_rust_types() {
        assert_eq!(NoPatch, NoPatch);
        assert_no_patch::<LockedField>();
    }
}
