use std::collections::{BTreeMap, BTreeSet};

use crate::mindustry::{
    ai::{unit_command::UnitCommand, unit_stance::UnitStance},
    content::{blocks::BlockDef, ContentCatalog},
    ctype::{Content, ContentId, ContentType},
    io::save::{ContentHeaderEntry, ContentHeaderSnapshot},
    r#type::{Item, Liquid, SectorPreset, StatusEffect, TeamEntry, UnitType},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRecord {
    pub content_type: ContentType,
    pub id: ContentId,
    pub mappable_name: Option<String>,
}

impl ContentRecord {
    pub fn content(content_type: ContentType, id: ContentId) -> Self {
        Self {
            content_type,
            id,
            mappable_name: None,
        }
    }

    pub fn mappable(content_type: ContentType, id: ContentId, name: impl Into<String>) -> Self {
        Self {
            content_type,
            id,
            mappable_name: Some(name.into()),
        }
    }

    pub fn is_mappable(&self) -> bool {
        self.mappable_name.is_some()
    }

    pub fn name(&self) -> Option<&str> {
        self.mappable_name.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentBucket {
    entries: Vec<ContentRecord>,
    names: BTreeMap<String, ContentId>,
}

impl ContentBucket {
    pub fn entries(&self) -> &[ContentRecord] {
        &self.entries
    }

    pub fn names(&self) -> &BTreeMap<String, ContentId> {
        &self.names
    }

    pub fn first(&self) -> Option<&ContentRecord> {
        self.entries.first()
    }

    pub fn get(&self, id: ContentId) -> Option<&ContentRecord> {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.entries.get(index))
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ContentRecord> {
        self.names.get(name).and_then(|id| self.get(*id))
    }

    fn push(&mut self, record: ContentRecord) -> Result<(), ContentLoaderError> {
        if let Some(name) = record.name() {
            if self.names.contains_key(name) {
                return Err(ContentLoaderError::DuplicateName {
                    content_type: record.content_type,
                    name: name.to_string(),
                });
            }
            self.names.insert(name.to_string(), record.id);
        }
        self.entries.push(record);
        Ok(())
    }

    fn pop_if_last(&mut self, record: &ContentRecord) -> Option<ContentRecord> {
        if self.entries.last() != Some(record) {
            return None;
        }
        let removed = self.entries.pop()?;
        if let Some(name) = removed.name() {
            self.names.remove(name);
        }
        Some(removed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryContentMapper {
    mapped: Vec<Vec<Option<ContentRecord>>>,
}

impl TemporaryContentMapper {
    pub fn empty() -> Self {
        Self {
            mapped: vec![Vec::new(); ContentType::ALL.len()],
        }
    }

    pub fn set(
        &mut self,
        content_type: ContentType,
        records: Vec<Option<ContentRecord>>,
    ) -> &mut Self {
        self.mapped[content_type.ordinal() as usize] = records;
        self
    }

    pub fn get(&self, content_type: ContentType, id: ContentId) -> TemporaryLookup<'_> {
        let mapped = &self.mapped[content_type.ordinal() as usize];
        if mapped.is_empty() {
            return TemporaryLookup::UnmappedType;
        }
        if id < 0 {
            return TemporaryLookup::InvalidId;
        }
        match mapped.get(id as usize).and_then(Option::as_ref) {
            Some(record) => TemporaryLookup::Mapped(record),
            None => TemporaryLookup::MissingMappedSlot,
        }
    }

    pub fn mapped_names(&self, content_type: ContentType) -> Vec<Option<&str>> {
        self.mapped[content_type.ordinal() as usize]
            .iter()
            .map(|record| record.as_ref().and_then(ContentRecord::name))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporaryLookup<'a> {
    UnmappedType,
    InvalidId,
    MissingMappedSlot,
    Mapped(&'a ContentRecord),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentInitialization {
    Init,
    PostInit,
    LoadIcon,
    Load,
}

#[derive(Debug, Clone)]
pub struct ContentLoader {
    catalog: ContentCatalog,
    buckets: Vec<ContentBucket>,
    name_map: BTreeMap<String, ContentRecord>,
    temporary_mapper: Option<TemporaryContentMapper>,
    current_mod: Option<String>,
    last_added: Option<ContentRecord>,
    initialization: BTreeSet<ContentInitialization>,
}

impl ContentLoader {
    pub fn empty() -> Self {
        Self {
            catalog: ContentCatalog::default(),
            buckets: vec![ContentBucket::default(); ContentType::ALL.len()],
            name_map: BTreeMap::new(),
            temporary_mapper: None,
            current_mod: None,
            last_added: None,
            initialization: BTreeSet::new(),
        }
    }

    pub fn create_base_content() -> Result<Self, ContentLoaderError> {
        Self::from_catalog(ContentCatalog::load_base_content())
    }

    pub fn create_base_content_or_panic() -> Self {
        Self::create_base_content()
            .expect("base content catalog must satisfy ContentLoader invariants")
    }

    pub fn from_catalog(catalog: ContentCatalog) -> Result<Self, ContentLoaderError> {
        let mut loader = Self {
            catalog,
            buckets: vec![ContentBucket::default(); ContentType::ALL.len()],
            name_map: BTreeMap::new(),
            temporary_mapper: None,
            current_mod: None,
            last_added: None,
            initialization: BTreeSet::new(),
        };
        loader.rebuild_indexes()?;
        Ok(loader)
    }

    pub fn catalog(&self) -> &ContentCatalog {
        &self.catalog
    }

    pub fn catalog_mut(&mut self) -> &mut ContentCatalog {
        &mut self.catalog
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn set_current_mod(&mut self, current_mod: Option<impl Into<String>>) {
        self.current_mod = current_mod.map(Into::into);
    }

    pub fn transform_name(&self, name: &str) -> String {
        self.current_mod
            .as_ref()
            .map(|module| format!("{module}-{name}"))
            .unwrap_or_else(|| name.to_string())
    }

    pub fn get_last_added(&self) -> Option<&ContentRecord> {
        self.last_added.as_ref()
    }

    pub fn remove_last(&mut self) -> Option<ContentRecord> {
        let last = self.last_added.take()?;
        let bucket = self.bucket_mut(last.content_type);
        let removed = bucket.pop_if_last(&last)?;
        if let Some(name) = removed.name() {
            if self.name_map.get(name) == Some(&removed) {
                self.name_map.remove(name);
            }
        }
        Some(removed)
    }

    pub fn handle_content_record(
        &mut self,
        record: ContentRecord,
    ) -> Result<(), ContentLoaderError> {
        let content_type = record.content_type;
        self.bucket_mut(content_type).push(record.clone())?;
        if let Some(name) = record.name() {
            self.name_map.insert(name.to_string(), record.clone());
        }
        self.last_added = Some(record);
        Ok(())
    }

    pub fn get_by(&self, content_type: ContentType) -> &[ContentRecord] {
        self.bucket(content_type).entries()
    }

    pub fn get_names_by(&self, content_type: ContentType) -> &BTreeMap<String, ContentId> {
        self.bucket(content_type).names()
    }

    pub fn each(&self) -> impl Iterator<Item = &ContentRecord> {
        self.buckets.iter().flat_map(|bucket| bucket.entries.iter())
    }

    pub fn by_name(&self, name: &str) -> Option<&ContentRecord> {
        self.name_map.get(name)
    }

    pub fn get_by_name(&self, content_type: ContentType, name: &str) -> Option<&ContentRecord> {
        self.bucket(content_type).get_by_name(name)
    }

    pub fn get_by_id(&self, content_type: ContentType, id: ContentId) -> Option<&ContentRecord> {
        if let Some(mapper) = &self.temporary_mapper {
            match mapper.get(content_type, id) {
                TemporaryLookup::Mapped(record) => return Some(record),
                TemporaryLookup::InvalidId => return None,
                TemporaryLookup::MissingMappedSlot => return self.bucket(content_type).first(),
                TemporaryLookup::UnmappedType => {}
            }
        }
        self.bucket(content_type).get(id)
    }

    pub fn item(&self, id: ContentId) -> Option<&Item> {
        self.catalog.item_by_id(id)
    }

    pub fn item_by_name(&self, name: &str) -> Option<&Item> {
        self.catalog.item_by_name(name)
    }

    pub fn items(&self) -> &[Item] {
        &self.catalog.items
    }

    pub fn block(&self, id: ContentId) -> Option<&BlockDef> {
        self.catalog.blocks.get(id)
    }

    pub fn block_by_name(&self, name: &str) -> Option<&BlockDef> {
        self.catalog.blocks.get_by_name(name)
    }

    pub fn blocks(&self) -> impl Iterator<Item = &BlockDef> {
        self.catalog.blocks.iter()
    }

    pub fn liquid(&self, id: ContentId) -> Option<&Liquid> {
        self.catalog.liquid_by_id(id)
    }

    pub fn liquid_by_name(&self, name: &str) -> Option<&Liquid> {
        self.catalog.liquid_by_name(name)
    }

    pub fn liquids(&self) -> &[Liquid] {
        &self.catalog.liquids
    }

    pub fn status_effect_by_name(&self, name: &str) -> Option<&StatusEffect> {
        self.catalog.status_effect_by_name(name)
    }

    pub fn status_effects(&self) -> &[StatusEffect] {
        &self.catalog.status_effects
    }

    pub fn weather(
        &self,
        id: ContentId,
    ) -> Option<&crate::mindustry::content::weathers::WeatherContent> {
        self.catalog.weather_by_id(id)
    }

    pub fn weather_by_name(
        &self,
        name: &str,
    ) -> Option<&crate::mindustry::content::weathers::WeatherContent> {
        self.catalog.weather_by_name(name)
    }

    pub fn weathers(&self) -> &[crate::mindustry::content::weathers::WeatherContent] {
        &self.catalog.weathers
    }

    pub fn unit(&self, id: ContentId) -> Option<&UnitType> {
        self.catalog.unit_by_id(id)
    }

    pub fn unit_by_name(&self, name: &str) -> Option<&UnitType> {
        self.catalog.unit_by_name(name)
    }

    pub fn units(&self) -> &[UnitType] {
        &self.catalog.units
    }

    pub fn sector_by_name(&self, name: &str) -> Option<&SectorPreset> {
        self.catalog.sector_by_name(name)
    }

    pub fn sectors(&self) -> &[SectorPreset] {
        &self.catalog.sectors
    }

    pub fn team_entry_by_name(&self, name: &str) -> Option<&TeamEntry> {
        self.catalog.team_entry_by_name(name)
    }

    pub fn team_entries(&self) -> &[TeamEntry] {
        &self.catalog.team_entries
    }

    pub fn unit_command(&self, id: ContentId) -> Option<&UnitCommand> {
        self.catalog.unit_command_by_id(id)
    }

    pub fn unit_command_by_name(&self, name: &str) -> Option<&UnitCommand> {
        self.catalog.unit_command_by_name(name)
    }

    pub fn unit_commands(&self) -> &[UnitCommand] {
        &self.catalog.unit_commands
    }

    pub fn unit_stance(&self, id: ContentId) -> Option<&UnitStance> {
        self.catalog.unit_stance_by_id(id)
    }

    pub fn unit_stance_by_name(&self, name: &str) -> Option<&UnitStance> {
        self.catalog.unit_stance_by_name(name)
    }

    pub fn unit_stances(&self) -> &[UnitStance] {
        &self.catalog.unit_stances
    }

    pub fn set_temporary_mapper(&mut self, temporary_mapper: Option<TemporaryContentMapper>) {
        self.temporary_mapper = temporary_mapper;
    }

    pub fn clear_temporary_mapper(&mut self) {
        self.temporary_mapper = None;
    }

    pub fn temporary_mapper(&self) -> Option<&TemporaryContentMapper> {
        self.temporary_mapper.as_ref()
    }

    pub fn temporary_mapper_from_header(
        &self,
        snapshot: &ContentHeaderSnapshot,
        block_name_fallback: &BTreeMap<String, String>,
    ) -> Result<TemporaryContentMapper, ContentLoaderError> {
        let mut mapper = TemporaryContentMapper::empty();
        for entry in &snapshot.entries {
            let content_type = ContentType::from_ordinal(entry.content_type)
                .ok_or(ContentLoaderError::UnknownContentType(entry.content_type))?;
            let mapped = entry
                .names
                .iter()
                .map(|name| {
                    let resolved = if content_type == ContentType::Block {
                        block_name_fallback
                            .get(name)
                            .map(String::as_str)
                            .unwrap_or(name)
                    } else {
                        name
                    };
                    self.get_by_name(content_type, resolved).cloned()
                })
                .collect();
            mapper.set(content_type, mapped);
        }
        Ok(mapper)
    }

    pub fn read_content_header(
        &mut self,
        snapshot: &ContentHeaderSnapshot,
        block_name_fallback: &BTreeMap<String, String>,
    ) -> Result<(), ContentLoaderError> {
        let mapper = self.temporary_mapper_from_header(snapshot, block_name_fallback)?;
        self.set_temporary_mapper(Some(mapper));
        Ok(())
    }

    pub fn content_header_snapshot(&self) -> ContentHeaderSnapshot {
        let entries = ContentType::ALL
            .iter()
            .copied()
            .filter_map(|content_type| {
                let bucket = self.bucket(content_type);
                if !bucket.first().is_some_and(ContentRecord::is_mappable) {
                    return None;
                }
                Some(ContentHeaderEntry {
                    content_type: content_type.ordinal(),
                    names: bucket
                        .entries
                        .iter()
                        .filter_map(|record| record.name().map(str::to_string))
                        .collect(),
                })
            })
            .collect();
        ContentHeaderSnapshot { entries }
    }

    pub fn initialize(&mut self, phase: ContentInitialization) -> bool {
        self.initialization.insert(phase)
    }

    pub fn initialized(&self, phase: &ContentInitialization) -> bool {
        self.initialization.contains(phase)
    }

    pub fn validate_linear_ids(&self) -> Result<(), ContentLoaderError> {
        for content_type in ContentType::ALL {
            for (index, record) in self.bucket(content_type).entries.iter().enumerate() {
                if record.id != index as ContentId {
                    return Err(ContentLoaderError::OutOfOrderId {
                        content_type,
                        name: record.name().map(str::to_string),
                        expected: index as ContentId,
                        actual: record.id,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn content_counts(&self) -> BTreeMap<ContentType, usize> {
        ContentType::ALL
            .iter()
            .copied()
            .map(|content_type| (content_type, self.bucket(content_type).entries.len()))
            .collect()
    }

    fn rebuild_indexes(&mut self) -> Result<(), ContentLoaderError> {
        self.buckets = vec![ContentBucket::default(); ContentType::ALL.len()];
        self.name_map.clear();
        self.last_added = None;

        let mut records = self.records_from_catalog();
        records.sort_by_key(|record| (record.content_type, record.id));
        for record in records {
            self.handle_content_record(record)?;
        }
        self.validate_linear_ids()
    }

    fn records_from_catalog(&self) -> Vec<ContentRecord> {
        let mut out = Vec::new();

        out.extend(self.catalog.unit_commands.iter().map(|command| {
            ContentRecord::mappable(ContentType::UnitCommand, command.id(), command.name())
        }));
        out.extend(
            self.catalog
                .team_entries
                .iter()
                .map(|entry| ContentRecord::mappable(ContentType::Team, entry.id(), entry.name())),
        );
        out.extend(self.catalog.items.iter().map(|item| {
            ContentRecord::mappable(ContentType::Item, item.base.mappable.base.id, item.name())
        }));
        out.extend(self.catalog.unit_stances.iter().map(|stance| {
            ContentRecord::mappable(ContentType::UnitStance, stance.id(), stance.name())
        }));
        out.extend(self.catalog.status_effects.iter().map(|status| {
            ContentRecord::mappable(
                ContentType::Status,
                status.base.mappable.base.id,
                status.name(),
            )
        }));
        out.extend(self.catalog.liquids.iter().map(|liquid| {
            ContentRecord::mappable(
                ContentType::Liquid,
                liquid.base.mappable.base.id,
                liquid.name(),
            )
        }));
        out.extend(
            self.catalog
                .bullets
                .iter()
                .map(|bullet| ContentRecord::content(ContentType::Bullet, bullet.id())),
        );
        out.extend(
            self.catalog
                .units
                .iter()
                .map(|unit| ContentRecord::mappable(ContentType::Unit, unit.id(), unit.name())),
        );
        out.extend(self.catalog.blocks.iter().map(|block| {
            ContentRecord::mappable(
                ContentType::Block,
                block.base().id,
                block.base().name.as_str(),
            )
        }));
        out.extend(self.catalog.weathers.iter().map(|weather| {
            ContentRecord::mappable(ContentType::Weather, weather.id(), weather.name())
        }));
        out.extend(self.catalog.planets.iter().map(|planet| {
            ContentRecord::mappable(ContentType::Planet, planet.id(), planet.name())
        }));
        out.extend(self.catalog.sectors.iter().map(|sector| {
            ContentRecord::mappable(ContentType::Sector, sector.id(), sector.name.as_str())
        }));

        out
    }

    fn bucket(&self, content_type: ContentType) -> &ContentBucket {
        &self.buckets[content_type.ordinal() as usize]
    }

    fn bucket_mut(&mut self, content_type: ContentType) -> &mut ContentBucket {
        &mut self.buckets[content_type.ordinal() as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentLoaderError {
    UnknownContentType(u8),
    DuplicateName {
        content_type: ContentType,
        name: String,
    },
    OutOfOrderId {
        content_type: ContentType,
        name: Option<String>,
        expected: ContentId,
        actual: ContentId,
    },
}

impl std::fmt::Display for ContentLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownContentType(content_type) => {
                write!(f, "unknown content type ordinal {content_type}")
            }
            Self::DuplicateName { content_type, name } => {
                write!(
                    f,
                    "duplicate {:?} mappable content name '{}'",
                    content_type, name
                )
            }
            Self::OutOfOrderId {
                content_type,
                name,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "out-of-order {:?} content id for {:?}: expected {}, got {}",
                    content_type, name, expected, actual
                )
            }
        }
    }
}

impl std::error::Error for ContentLoaderError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::save::{read_content_header_snapshot, write_content_header_snapshot};

    #[test]
    fn content_loader_builds_base_indexes_like_java_content_loader() {
        let loader = ContentLoader::create_base_content().unwrap();
        loader.validate_linear_ids().unwrap();

        assert_eq!(loader.get_by(ContentType::Item)[0].name(), Some("copper"));
        assert_eq!(loader.get_by(ContentType::Block)[0].name(), Some("air"));
        assert_eq!(
            loader.get_by(ContentType::UnitCommand)[0].name(),
            Some("move")
        );
        assert_eq!(
            loader.get_by(ContentType::UnitStance)[0].name(),
            Some("stop")
        );
        assert_eq!(
            loader.get_by_id(ContentType::Liquid, 0).unwrap().name(),
            Some("water")
        );
        assert_eq!(
            loader.get_by_name(ContentType::Unit, "flare").unwrap().id,
            15
        );
        assert_eq!(
            loader.by_name("router").unwrap().content_type,
            ContentType::Block
        );
        assert!(loader.by_name("damageLightning").is_none());
        assert_eq!(
            loader.get_by_id(ContentType::Bullet, 1).unwrap().name(),
            None
        );

        assert!(loader.team_entries().is_empty());
        assert!(loader.get_by(ContentType::Team).is_empty());
    }

    #[test]
    fn content_loader_header_snapshot_uses_generic_mappable_buckets() {
        let loader = ContentLoader::create_base_content().unwrap();
        let snapshot = loader.content_header_snapshot();
        let types: Vec<u8> = snapshot
            .entries
            .iter()
            .map(|entry| entry.content_type)
            .collect();

        assert_eq!(
            types,
            vec![
                ContentType::Item.ordinal(),
                ContentType::Block.ordinal(),
                ContentType::Liquid.ordinal(),
                ContentType::Status.ordinal(),
                ContentType::Unit.ordinal(),
                ContentType::Weather.ordinal(),
                ContentType::Sector.ordinal(),
                ContentType::Planet.ordinal(),
                ContentType::UnitCommand.ordinal(),
                ContentType::UnitStance.ordinal(),
            ]
        );
        assert!(!types.contains(&ContentType::Bullet.ordinal()));
        assert!(!types.contains(&ContentType::Team.ordinal()));
        assert_eq!(snapshot.entries[0].names[0], "copper");
        assert_eq!(snapshot.entries[0].names[8], "scrap");
        assert_eq!(snapshot.entries[1].names[0], "air");

        let mut bytes = Vec::new();
        write_content_header_snapshot(&mut bytes, &snapshot).unwrap();
        let decoded = read_content_header_snapshot(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn temporary_mapper_matches_java_read_content_header_fallback_rules() {
        let mut loader = ContentLoader::create_base_content().unwrap();
        let snapshot = ContentHeaderSnapshot {
            entries: vec![
                ContentHeaderEntry {
                    content_type: ContentType::Block.ordinal(),
                    names: vec![
                        "legacy-router".into(),
                        "unknown-block".into(),
                        "conveyor".into(),
                    ],
                },
                ContentHeaderEntry {
                    content_type: ContentType::Item.ordinal(),
                    names: vec!["copper".into()],
                },
            ],
        };
        let fallback = BTreeMap::from([("legacy-router".into(), "router".into())]);

        loader.read_content_header(&snapshot, &fallback).unwrap();

        assert_eq!(
            loader.get_by_id(ContentType::Block, 0).unwrap().name(),
            Some("router")
        );
        assert_eq!(
            loader.get_by_id(ContentType::Block, 1).unwrap().name(),
            Some("air")
        );
        assert_eq!(
            loader.get_by_id(ContentType::Block, 2).unwrap().name(),
            Some("conveyor")
        );
        assert!(loader.get_by_id(ContentType::Block, -1).is_none());
        assert_eq!(
            loader.get_by_id(ContentType::Item, 0).unwrap().name(),
            Some("copper")
        );
        assert_eq!(
            loader.get_by_id(ContentType::Liquid, 0).unwrap().name(),
            Some("water")
        );
        assert!(loader.get_by_id(ContentType::Liquid, 999).is_none());
    }

    #[test]
    fn content_loader_tracks_current_mod_last_added_and_duplicate_names() {
        let mut loader = ContentLoader::empty();
        loader.set_current_mod(Some("demo"));
        assert_eq!(loader.transform_name("block"), "demo-block");

        let copper = ContentRecord::mappable(ContentType::Item, 0, "copper");
        loader.handle_content_record(copper.clone()).unwrap();
        assert_eq!(loader.get_last_added(), Some(&copper));
        assert_eq!(loader.by_name("copper"), Some(&copper));

        let duplicate =
            loader.handle_content_record(ContentRecord::mappable(ContentType::Item, 1, "copper"));
        assert!(matches!(
            duplicate,
            Err(ContentLoaderError::DuplicateName {
                content_type: ContentType::Item,
                ..
            })
        ));

        assert_eq!(loader.remove_last(), Some(copper));
        assert!(loader.by_name("copper").is_none());
        assert!(loader.get_by(ContentType::Item).is_empty());
    }
}
