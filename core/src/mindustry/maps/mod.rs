use std::{collections::BTreeMap, io, io::Read, path::Path};

use crate::mindustry::{
    game::{rules::GamemodeApplier, Gamemode, Rules},
    io::{read_deflated_save_meta, SaveMeta},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDescriptor {
    pub file: String,
    pub tags: BTreeMap<String, String>,
    pub custom: bool,
    pub version: i32,
    pub workshop: bool,
    pub width: i32,
    pub height: i32,
    pub build: i32,
    pub teams: Vec<i32>,
    pub spawns: i32,
    pub mod_name: Option<String>,
}

impl MapDescriptor {
    pub fn new(
        file: impl Into<String>,
        width: i32,
        height: i32,
        tags: BTreeMap<String, String>,
        custom: bool,
        version: i32,
        build: i32,
    ) -> Self {
        Self {
            file: file.into(),
            tags,
            custom,
            version,
            workshop: false,
            width,
            height,
            build,
            teams: Vec::new(),
            spawns: 0,
            mod_name: None,
        }
    }

    pub fn from_save_meta(file: impl Into<String>, meta: &SaveMeta) -> Self {
        Self {
            file: file.into(),
            tags: meta.tags.clone(),
            custom: true,
            version: meta.version,
            workshop: false,
            width: 0,
            height: 0,
            build: meta.build,
            teams: Vec::new(),
            spawns: 0,
            mod_name: None,
        }
    }

    pub fn from_deflated_save_reader(file: impl Into<String>, read: impl Read) -> io::Result<Self> {
        let meta = read_deflated_save_meta(read)?;
        Ok(Self::from_save_meta(file, &meta))
    }

    pub fn name(&self) -> &str {
        self.tag("name")
    }

    pub fn author(&self) -> &str {
        self.tag("author")
    }

    pub fn description(&self) -> &str {
        self.tag("description")
    }

    pub fn tag(&self, name: &str) -> &str {
        if self.has_tag(name) {
            self.tags.get(name).unwrap()
        } else {
            "unknown"
        }
    }

    pub fn has_tag(&self, name: &str) -> bool {
        self.tags
            .get(name)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    }

    pub fn steam_id(&self) -> Option<&str> {
        self.tags.get("steamid").map(String::as_str)
    }

    pub fn preview_file(&self, preview_dir: &str) -> String {
        format!("{}/{}_v2.png", trim_slash(preview_dir), self.preview_stem())
    }

    pub fn cache_file(&self, preview_dir: &str) -> String {
        if self.workshop {
            format!(
                "{}/{}-workshop-cache.dat",
                trim_slash(preview_dir),
                self.preview_stem()
            )
        } else {
            format!(
                "{}/{}-cache_v2.dat",
                trim_slash(preview_dir),
                self.file_stem()
            )
        }
    }

    pub fn rules(&self) -> Rules {
        Rules::default()
    }

    pub fn apply_rules(&self, mode: Gamemode) -> Rules {
        let mut rules = Rules::default();
        mode.apply(&mut rules);
        rules
    }

    pub fn filters_tag(&self) -> Option<&str> {
        if self
            .tags
            .get("genfilters")
            .map(|s| s.is_empty())
            .unwrap_or(true)
            && self
                .tags
                .get("build")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(-1)
                < 83
            && self.tags.get("build").map(String::as_str) != Some("-1")
        {
            None
        } else {
            self.tags.get("genfilters").map(String::as_str)
        }
    }

    fn preview_stem(&self) -> String {
        if self.workshop {
            parent_name(&self.file).unwrap_or_else(|| self.file_stem())
        } else {
            self.file_stem()
        }
    }

    fn file_stem(&self) -> String {
        Path::new(&self.file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&self.file)
            .to_string()
    }
}

fn parent_name(path: &str) -> Option<String> {
    Path::new(path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(str::to_string)
}

fn trim_slash(path: &str) -> &str {
    path.trim_end_matches(['/', '\\'])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::{
        write_deflated_raw_save_envelope, write_string_map, RawSaveEnvelope, SaveRegion,
        LATEST_SAVE_VERSION,
    };

    #[test]
    fn descriptor_tags_and_preview_paths_match_java_rules() {
        let mut tags = BTreeMap::new();
        tags.insert("name".into(), "Map".into());
        tags.insert("author".into(), "Author".into());
        let map = MapDescriptor::new("maps/foo.msav", 10, 20, tags, true, 11, 157);
        assert_eq!(map.name(), "Map");
        assert_eq!(map.author(), "Author");
        assert_eq!(map.description(), "unknown");
        assert_eq!(map.preview_file("previews"), "previews/foo_v2.png");
        assert_eq!(map.cache_file("previews"), "previews/foo-cache_v2.dat");
    }

    #[test]
    fn workshop_paths_use_parent_directory_name() {
        let mut map = MapDescriptor::new(
            "workshop/123/map.msav",
            0,
            0,
            BTreeMap::new(),
            true,
            11,
            157,
        );
        map.workshop = true;
        assert_eq!(map.preview_file("previews"), "previews/123_v2.png");
        assert_eq!(
            map.cache_file("previews"),
            "previews/123-workshop-cache.dat"
        );
    }

    #[test]
    fn descriptor_can_be_created_from_java_deflated_save_meta() {
        let mut tags = BTreeMap::new();
        tags.insert("name".into(), "Deflated Map".into());
        tags.insert("author".into(), "Rust".into());
        tags.insert("version".into(), "11".into());
        tags.insert("build".into(), "1574".into());
        tags.insert("mapname".into(), "Deflated Map".into());

        let mut meta_payload = Vec::new();
        write_string_map(&mut meta_payload, &tags).unwrap();
        let mut envelope = RawSaveEnvelope::new(LATEST_SAVE_VERSION);
        envelope.set(SaveRegion::Meta, meta_payload).unwrap();

        let mut deflated = Vec::new();
        write_deflated_raw_save_envelope(&mut deflated, &envelope).unwrap();
        let descriptor =
            MapDescriptor::from_deflated_save_reader("maps/deflated.msav", deflated.as_slice())
                .unwrap();

        assert_eq!(descriptor.file, "maps/deflated.msav");
        assert_eq!(descriptor.name(), "Deflated Map");
        assert_eq!(descriptor.author(), "Rust");
        assert_eq!(descriptor.version, 11);
        assert_eq!(descriptor.build, 1574);
    }
}
