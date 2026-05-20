pub mod blocks;
pub mod items;
pub mod liquids;
pub mod status_effects;
pub mod unit_types;
pub mod weathers;

use crate::mindustry::{
    ctype::{Content, ContentId, ContentType},
    io::save::{ContentHeaderEntry, ContentHeaderSnapshot},
    r#type::{Item, Liquid, StatusEffect, UnitType},
};

#[derive(Debug, Clone, Default)]
pub struct ContentCatalog {
    pub blocks: blocks::BlockRegistry,
    pub items: Vec<Item>,
    pub liquids: Vec<Liquid>,
    pub status_effects: Vec<StatusEffect>,
    pub units: Vec<UnitType>,
    pub weathers: Vec<weathers::WeatherContent>,
}

impl ContentCatalog {
    pub fn load_base_content() -> Self {
        let items = items::load();
        let liquids = liquids::load();
        let blocks = blocks::load(&items, &liquids);
        Self {
            blocks,
            items,
            liquids,
            status_effects: status_effects::load(),
            units: unit_types::load(),
            weathers: weathers::load(),
        }
    }

    pub fn content_header_snapshot(&self) -> ContentHeaderSnapshot {
        ContentHeaderSnapshot {
            entries: vec![
                ContentHeaderEntry {
                    content_type: ContentType::Item.ordinal(),
                    names: self
                        .items
                        .iter()
                        .map(|item| item.base.mappable.name.clone())
                        .collect(),
                },
                ContentHeaderEntry {
                    content_type: ContentType::Block.ordinal(),
                    names: self
                        .blocks
                        .iter()
                        .map(|block| block.base().name.clone())
                        .collect(),
                },
                ContentHeaderEntry {
                    content_type: ContentType::Liquid.ordinal(),
                    names: self
                        .liquids
                        .iter()
                        .map(|liquid| liquid.base.mappable.name.clone())
                        .collect(),
                },
                ContentHeaderEntry {
                    content_type: ContentType::Status.ordinal(),
                    names: self
                        .status_effects
                        .iter()
                        .map(|status| status.base.mappable.name.clone())
                        .collect(),
                },
                ContentHeaderEntry {
                    content_type: ContentType::Unit.ordinal(),
                    names: self
                        .units
                        .iter()
                        .map(|unit| unit.base.mappable.name.clone())
                        .collect(),
                },
                ContentHeaderEntry {
                    content_type: ContentType::Weather.ordinal(),
                    names: self
                        .weathers
                        .iter()
                        .map(|weather| weather.name().to_string())
                        .collect(),
                },
            ],
        }
    }

    pub fn item_by_name(&self, name: &str) -> Option<&Item> {
        self.items
            .iter()
            .find(|item| item.base.mappable.name.as_str() == name)
    }

    pub fn liquid_by_name(&self, name: &str) -> Option<&Liquid> {
        self.liquids
            .iter()
            .find(|liquid| liquid.base.mappable.name.as_str() == name)
    }

    pub fn status_effect_by_name(&self, name: &str) -> Option<&StatusEffect> {
        self.status_effects
            .iter()
            .find(|status| status.base.mappable.name.as_str() == name)
    }

    pub fn weather_by_name(&self, name: &str) -> Option<&weathers::WeatherContent> {
        self.weathers.iter().find(|weather| weather.name() == name)
    }

    pub fn unit_by_name(&self, name: &str) -> Option<&UnitType> {
        self.units
            .iter()
            .find(|unit| unit.base.mappable.name.as_str() == name)
    }

    pub fn item_by_id(&self, id: ContentId) -> Option<&Item> {
        self.items
            .iter()
            .find(|item| item.base.mappable.base.id == id)
    }

    pub fn liquid_by_id(&self, id: ContentId) -> Option<&Liquid> {
        self.liquids
            .iter()
            .find(|liquid| liquid.base.mappable.base.id == id)
    }

    pub fn status_effect_by_id(&self, id: ContentId) -> Option<&StatusEffect> {
        self.status_effects
            .iter()
            .find(|status| status.base.mappable.base.id == id)
    }

    pub fn weather_by_id(&self, id: ContentId) -> Option<&weathers::WeatherContent> {
        self.weathers.iter().find(|weather| weather.id() == id)
    }

    pub fn unit_by_id(&self, id: ContentId) -> Option<&UnitType> {
        self.units
            .iter()
            .find(|unit| unit.base.mappable.base.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::ContentCatalog;
    use crate::mindustry::{
        ctype::{Content, ContentType},
        io::save::{read_content_header_snapshot, write_content_header_snapshot},
    };

    #[test]
    fn catalog_builds_content_header_snapshot_in_content_type_order() {
        let catalog = ContentCatalog::load_base_content();
        let snapshot = catalog.content_header_snapshot();
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
            ]
        );
        assert_eq!(snapshot.entries[0].names[0], "scrap");
        assert_eq!(snapshot.entries[1].names[0], "air");
        assert_eq!(snapshot.entries[2].names[0], "water");
        assert_eq!(snapshot.entries[3].names[0], "none");
        assert_eq!(snapshot.entries[4].names[0], "dagger");
        assert_eq!(snapshot.entries[5].names[0], "snowing");
    }

    #[test]
    fn catalog_content_header_snapshot_roundtrips_through_save_codec() {
        let snapshot = ContentCatalog::load_base_content().content_header_snapshot();
        let mut bytes = Vec::new();
        write_content_header_snapshot(&mut bytes, &snapshot).unwrap();
        let decoded = read_content_header_snapshot(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn catalog_lookup_helpers_use_stable_content_ids_and_names() {
        let catalog = ContentCatalog::load_base_content();
        let copper = catalog.item_by_name("copper").unwrap();
        assert_eq!(
            catalog
                .item_by_id(copper.base.mappable.base.id)
                .unwrap()
                .base
                .mappable
                .name,
            "copper"
        );
        assert_eq!(
            catalog
                .liquid_by_name("cryofluid")
                .unwrap()
                .base
                .mappable
                .name,
            "cryofluid"
        );
        assert_eq!(
            catalog.liquid_by_id(0).unwrap().base.mappable.name.as_str(),
            "water"
        );
        assert_eq!(
            catalog
                .status_effect_by_name("wet")
                .unwrap()
                .base
                .mappable
                .name,
            "wet"
        );
        assert!(catalog.status_effect_by_id(999).is_none());
        assert_eq!(catalog.weather_by_name("rain").unwrap().id(), 1);
        assert_eq!(catalog.weather_by_id(2).unwrap().name(), "sandstorm");
        assert!(catalog.weather_by_id(999).is_none());
        assert_eq!(catalog.unit_by_name("flare").unwrap().id(), 15);
        assert_eq!(catalog.unit_by_id(60).unwrap().name(), "assembly-drone");
        assert!(catalog.unit_by_id(999).is_none());
    }
}
