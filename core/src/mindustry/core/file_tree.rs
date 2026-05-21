//! Mod-aware asset file resolver mirroring upstream `mindustry.core.FileTree`.
//!
//! Java delegates actual file handles and asset loading to Arc. This Rust port
//! keeps the deterministic path-resolution, cache and audio-extension selection
//! semantics so higher layers can plug in concrete filesystem/asset backends
//! later.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetFile {
    pub path: String,
    pub exists: bool,
}

impl AssetFile {
    pub fn new(path: impl Into<String>, exists: bool) -> Self {
        Self {
            path: normalize_path(path),
            exists,
        }
    }

    pub fn missing(path: impl Into<String>) -> Self {
        Self::new(path, false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundAsset {
    pub name: String,
    pub path: Option<String>,
}

impl SoundAsset {
    pub fn none() -> Self {
        Self {
            name: "none".into(),
            path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicAsset {
    pub name: String,
    pub path: Option<String>,
}

impl MusicAsset {
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileTree {
    files: BTreeMap<String, AssetFile>,
    loaded_sounds: BTreeMap<String, SoundAsset>,
    loaded_music: BTreeMap<String, MusicAsset>,
    headless: bool,
}

impl FileTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_headless(headless: bool) -> Self {
        Self {
            headless,
            ..Self::default()
        }
    }

    pub fn set_headless(&mut self, headless: bool) {
        self.headless = headless;
    }

    pub fn is_headless(&self) -> bool {
        self.headless
    }

    pub fn add_file(&mut self, path: impl Into<String>, file: AssetFile) {
        self.files.insert(normalize_path(path), file);
    }

    /// Gets an asset file.
    pub fn get(&self, path: &str) -> AssetFile {
        self.get_safe(path, false)
    }

    /// Gets an asset file, with `safe` preserving the Java call shape.
    pub fn get_safe(&self, path: &str, _safe: bool) -> AssetFile {
        let path = normalize_path(path);
        if let Some(file) = self.files.get(&path) {
            file.clone()
        } else {
            let slash_path = format!("/{path}");
            self.files
                .get(&slash_path)
                .cloned()
                .unwrap_or_else(|| AssetFile::missing(path))
        }
    }

    /// Clears all mod files.
    pub fn clear(&mut self) {
        self.files.clear();
    }

    pub fn resolve(&self, file_name: &str) -> AssetFile {
        self.get(file_name)
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn sound_cache_len(&self) -> usize {
        self.loaded_sounds.len()
    }

    pub fn music_cache_len(&self) -> usize {
        self.loaded_music.len()
    }

    pub fn load_sound(&mut self, sound_name: &str) -> SoundAsset {
        if self.headless {
            return SoundAsset::none();
        }

        if let Some(sound) = self.loaded_sounds.get(sound_name) {
            return sound.clone();
        }

        let name = format!("sounds/{sound_name}");
        let sound = SoundAsset {
            name: sound_name.to_string(),
            path: self.get_audio_path(&name),
        };
        self.loaded_sounds
            .insert(sound_name.to_string(), sound.clone());
        sound
    }

    pub fn load_music(&mut self, music_name: &str) -> MusicAsset {
        if self.headless {
            return MusicAsset::empty(music_name);
        }

        if let Some(music) = self.loaded_music.get(music_name) {
            return music.clone();
        }

        let name = format!("music/{music_name}");
        let music = MusicAsset {
            name: music_name.to_string(),
            path: self.get_audio_path(&name),
        };
        self.loaded_music
            .insert(music_name.to_string(), music.clone());
        music
    }

    pub fn get_audio_path(&self, name: &str) -> Option<String> {
        let ogg = format!("{name}.ogg");
        let mp3 = format!("{name}.mp3");
        if self.get(&ogg).exists {
            Some(ogg)
        } else if self.get(&mp3).exists {
            Some(mp3)
        } else {
            None
        }
    }
}

pub fn normalize_path(path: impl Into<String>) -> String {
    path.into().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_tree_add_file_normalizes_backslashes_and_resolves_leading_slash() {
        let mut tree = FileTree::new();
        tree.add_file(
            "sprites\\blocks\\router.png",
            AssetFile::new("mods/test/router.png", true),
        );
        tree.add_file(
            "/sounds/click.ogg",
            AssetFile::new("mods/test/click.ogg", true),
        );

        assert_eq!(
            tree.get("sprites/blocks/router.png").path,
            "mods/test/router.png"
        );
        assert!(tree.get("sounds/click.ogg").exists);
        assert_eq!(tree.get("missing.png"), AssetFile::missing("missing.png"));
        assert_eq!(
            tree.resolve("sprites\\blocks\\router.png").path,
            "mods/test/router.png"
        );
    }

    #[test]
    fn clear_removes_only_mod_file_overrides_like_java() {
        let mut tree = FileTree::new();
        tree.add_file("a", AssetFile::new("a", true));
        tree.load_sound("missing");

        assert_eq!(tree.file_count(), 1);
        assert_eq!(tree.sound_cache_len(), 1);

        tree.clear();

        assert_eq!(tree.file_count(), 0);
        assert_eq!(tree.sound_cache_len(), 1);
        assert!(!tree.get("a").exists);
    }

    #[test]
    fn audio_path_prefers_ogg_then_mp3_and_caches_results() {
        let mut tree = FileTree::new();
        tree.add_file("sounds/laser.mp3", AssetFile::new("laser.mp3", true));
        tree.add_file("music/theme.ogg", AssetFile::new("theme.ogg", true));
        tree.add_file("music/theme.mp3", AssetFile::new("theme.mp3", true));

        assert_eq!(
            tree.get_audio_path("sounds/laser"),
            Some("sounds/laser.mp3".into())
        );
        assert_eq!(
            tree.load_sound("laser").path,
            Some("sounds/laser.mp3".into())
        );
        assert_eq!(tree.sound_cache_len(), 1);
        assert_eq!(
            tree.load_sound("laser").path,
            Some("sounds/laser.mp3".into())
        );
        assert_eq!(tree.sound_cache_len(), 1);

        assert_eq!(
            tree.load_music("theme").path,
            Some("music/theme.ogg".into())
        );
        assert_eq!(tree.music_cache_len(), 1);
    }

    #[test]
    fn headless_audio_returns_empty_assets_without_caching() {
        let mut tree = FileTree::with_headless(true);
        tree.add_file("sounds/click.ogg", AssetFile::new("click.ogg", true));

        assert_eq!(tree.load_sound("click"), SoundAsset::none());
        assert_eq!(tree.sound_cache_len(), 0);

        assert_eq!(tree.load_music("theme"), MusicAsset::empty("theme"));
        assert_eq!(tree.music_cache_len(), 0);
    }
}
