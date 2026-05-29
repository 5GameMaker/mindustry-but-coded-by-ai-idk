use std::io;

use crate::mindustry::game::{read_schematic_base64, Schematic};

pub const BASIC_SHARD: &str = "basicShard";
pub const BASIC_FOUNDATION: &str = "basicFoundation";
pub const BASIC_NUCLEUS: &str = "basicNucleus";
pub const BASIC_BASTION: &str = "basicBastion";

const BASIC_SHARD_BASE64: &str = "bXNjaAF4nGNgZmBmZmDJS8xNZZDJKCkpKLbS16/MLy0p1UtK1XcNi/Q3cKwwyqkyYOBOSS1OLsosKMnMz2NgYGDLSUxKzSlmYIqOZWTgSs4vStUtzkgsSgFKMYIQkAAAhSEXTA==";
const BASIC_FOUNDATION_BASE64: &str = "bXNjaAF4nGNgYWBhZmDJS8xNZWBNSk3MK2bgTkktTi7KLCjJzM9jYGBgy0lMSs0pZmCKjmVk4E/OL0rVTcsvzUtJhMozghCQAACx6RHB";
const BASIC_NUCLEUS_BASE64: &str = "bXNjaAF4nA3CwQ2AIBAEwAXFjxRBA1ZkfCDcgwh3BiTG7iUzMDATZvaFYGOK7pPuLpYXa6QWarqfJAxVsGR/Um7Q+6Fgg1TauIdMvQFQgB7wAza8E4M=";
const BASIC_BASTION_BASE64: &str = "bXNjaAF4nGNgYWBhZmDJS8xNZWBNzMsEUtwpqcXJRZkFJZn5eQyClfmlCin5Cnn5JQqpFZnFJVwMbDmJSak5xQxM0bGMDDzJ+UWpukmJxWDVDAyMIAQkACMdFqE=";

#[derive(Debug, Clone, PartialEq)]
pub struct Loadout {
    pub name: String,
    pub schematic: Schematic,
}

impl Loadout {
    pub fn new(name: impl Into<String>, schematic: Schematic) -> Self {
        Self {
            name: name.into(),
            schematic,
        }
    }

    pub fn core_block_name(&self) -> Option<&str> {
        self.schematic
            .tiles
            .iter()
            .find(|tile| tile.block.starts_with("core-"))
            .map(|tile| tile.block.as_str())
    }
}

pub fn load() -> io::Result<Vec<Loadout>> {
    Ok(vec![
        read_loadout(BASIC_SHARD, BASIC_SHARD_BASE64)?,
        read_loadout(BASIC_FOUNDATION, BASIC_FOUNDATION_BASE64)?,
        read_loadout(BASIC_NUCLEUS, BASIC_NUCLEUS_BASE64)?,
        read_loadout(BASIC_BASTION, BASIC_BASTION_BASE64)?,
    ])
}

pub fn load_or_panic() -> Vec<Loadout> {
    load().expect("vanilla loadout schematic constants must decode")
}

fn read_loadout(name: &str, base64: &str) -> io::Result<Loadout> {
    read_schematic_base64(base64).map(|schematic| Loadout::new(name, schematic))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        game::{read_schematic_base64, write_schematic_base64},
        io::type_io::TypeValue,
    };

    #[test]
    fn vanilla_loadout_order_matches_upstream_static_fields() {
        let loadouts = load().unwrap();
        let names: Vec<_> = loadouts
            .iter()
            .map(|loadout| loadout.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec![BASIC_SHARD, BASIC_FOUNDATION, BASIC_NUCLEUS, BASIC_BASTION]
        );
    }

    #[test]
    fn vanilla_loadouts_decode_to_single_core_tile_schematics() {
        let loadouts = load().unwrap();
        let expected = [
            (BASIC_SHARD, 3, 3, "core-shard", 1, 1),
            (BASIC_FOUNDATION, 4, 4, "core-foundation", 1, 1),
            (BASIC_NUCLEUS, 5, 5, "core-nucleus", 2, 2),
            (BASIC_BASTION, 4, 4, "core-bastion", 1, 1),
        ];

        for (loadout, (name, width, height, block, x, y)) in loadouts.iter().zip(expected) {
            assert_eq!(loadout.name, name);
            assert_eq!(loadout.schematic.width, width);
            assert_eq!(loadout.schematic.height, height);
            assert_eq!(loadout.schematic.tiles.len(), 1);
            assert_eq!(loadout.core_block_name(), Some(block));

            let tile = &loadout.schematic.tiles[0];
            assert_eq!(tile.block, block);
            assert_eq!(tile.x, x);
            assert_eq!(tile.y, y);
            assert_eq!(tile.rotation, 0);
            assert_eq!(tile.config, TypeValue::Null);
            assert!(loadout.schematic.labels.is_empty());
            assert_eq!(
                loadout.schematic.tags.get("labels").map(String::as_str),
                Some("[]")
            );
        }
    }

    #[test]
    fn vanilla_loadout_schematic_metadata_matches_java_static_payloads() {
        let loadouts = load().unwrap();
        let expected = [
            (BASIC_SHARD, "https://youtu.be/EVYO0Ax2lz0", ""),
            (BASIC_FOUNDATION, "beans", ""),
            (BASIC_NUCLEUS, "did you know", "m"),
            (BASIC_BASTION, "anime", "you do not exist\n"),
        ];

        for (loadout, (name, schematic_name, description)) in loadouts.iter().zip(expected) {
            assert_eq!(loadout.name, name);
            assert_eq!(
                loadout.schematic.tags.get("name").map(String::as_str),
                Some(schematic_name)
            );
            assert_eq!(
                loadout
                    .schematic
                    .tags
                    .get("description")
                    .map(String::as_str),
                Some(description)
            );
            assert_eq!(
                loadout.schematic.tags.get("labels").map(String::as_str),
                Some("[]")
            );
        }
    }

    #[test]
    fn vanilla_loadout_schematics_roundtrip_through_base64_codec() {
        for loadout in load().unwrap() {
            let encoded = write_schematic_base64(&loadout.schematic).unwrap();
            let decoded = read_schematic_base64(&encoded).unwrap();
            assert_eq!(decoded.tiles, loadout.schematic.tiles);
            assert_eq!(decoded.labels, loadout.schematic.labels);
            assert_eq!(decoded.width, loadout.schematic.width);
            assert_eq!(decoded.height, loadout.schematic.height);
            assert_eq!(
                decoded.tags.get("contentMap").map(String::as_str),
                Some("{}")
            );
        }
    }
}
