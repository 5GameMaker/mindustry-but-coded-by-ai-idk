//! Mirrors upstream `mindustry.logic.GlobalVars`.

use crate::mindustry::{content::ContentCatalog, ctype::ContentType};

use std::sync::OnceLock;

use super::{
    logic_assembler::{named_logic_color_rgba, rgba_u32_to_double_bits, LogicValue},
    logic_team_from_name, LAccess, LogicAlign,
};

pub const LOGIC_CTRL_PROCESSOR: i32 = 1;
pub const LOGIC_CTRL_PLAYER: i32 = 2;
pub const LOGIC_CTRL_COMMAND: i32 = 3;
pub const LOOKABLE_CONTENT: [&str; 5] = ["block", "unit", "item", "liquid", "team"];
pub const WRITABLE_LOOKABLE_CONTENT: [&str; 4] = ["block", "unit", "item", "liquid"];
pub const LOOKABLE_CONTENT_TYPES: [ContentType; 5] = [
    ContentType::Block,
    ContentType::Unit,
    ContentType::Item,
    ContentType::Liquid,
    ContentType::Team,
];

pub fn logic_global_value(symbol: &str, privileged: bool) -> Option<LogicValue> {
    if matches!(
        symbol,
        "@server"
            | "@client"
            | "@clientLocale"
            | "@clientUnit"
            | "@clientName"
            | "@clientTeam"
            | "@clientMobile"
    ) && !privileged
    {
        return Some(LogicValue::Object(None));
    }

    match symbol {
        "the end" | "null" | "@wait" => Some(LogicValue::Object(None)),
        "@queries" if privileged => Some(LogicValue::Object(None)),
        "false" => Some(LogicValue::Number(0.0)),
        "true" => Some(LogicValue::Number(1.0)),
        "@pi" | "π" => Some(LogicValue::Number(std::f64::consts::PI)),
        "@e" => Some(LogicValue::Number(std::f64::consts::E)),
        "@degToRad" => Some(LogicValue::Number(std::f64::consts::PI / 180.0)),
        "@radToDeg" => Some(LogicValue::Number(180.0 / std::f64::consts::PI)),
        "@ctrlProcessor" => Some(LogicValue::Number(LOGIC_CTRL_PROCESSOR as f64)),
        "@ctrlPlayer" => Some(LogicValue::Number(LOGIC_CTRL_PLAYER as f64)),
        "@ctrlCommand" => Some(LogicValue::Number(LOGIC_CTRL_COMMAND as f64)),
        "@thisx" | "@thisy" | "@links" | "@ipt" | "@time" | "@tick" | "@second" | "@minute"
        | "@waveNumber" | "@waveTime" | "@mapw" | "@maph" | "@server" | "@client"
        | "@clientTeam" | "@clientMobile" => Some(LogicValue::Number(0.0)),
        "@clientLocale" | "@clientUnit" | "@clientName" => Some(LogicValue::Object(None)),
        _ => {
            if let Some(color_name) = symbol.strip_prefix("@color") {
                if !color_name.is_empty() {
                    if let Some(rgba) = named_logic_color_rgba(color_name) {
                        return Some(LogicValue::Number(rgba_u32_to_double_bits(rgba)));
                    }
                }
            }

            if let Some(name) = symbol.strip_prefix('@') {
                if LAccess::by_wire_name(name).is_some() || LogicAlign::by_name(name).is_some() {
                    return Some(LogicValue::Object(Some(symbol.to_string())));
                }

                if logic_known_global_content_name(name) || symbol.starts_with("@sfx-") {
                    return Some(LogicValue::Object(Some(symbol.to_string())));
                }
            }

            None
        }
    }
}

pub fn logic_known_global_content_name(name: &str) -> bool {
    static CATALOG: OnceLock<ContentCatalog> = OnceLock::new();
    const UNIT_NAMES: [&str; 58] = [
        "dagger", "mace", "fortress", "scepter", "reign", "nova", "pulsar", "quasar", "vela",
        "corvus", "crawler", "atrax", "spiroct", "arkyid", "toxopid", "flare", "horizon", "zenith",
        "antumbra", "eclipse", "mono", "poly", "mega", "quad", "oct", "risso", "minke", "bryde",
        "sei", "omura", "retusa", "oxynoe", "cyerce", "aegires", "navanax", "stell", "locus",
        "precept", "vanquish", "conquer", "merui", "cleroi", "anthicus", "tecta", "collaris",
        "elude", "avert", "obviate", "quell", "disrupt", "evoke", "incite", "emanate", "alpha",
        "beta", "gamma", "renale", "latum",
    ];
    const WEATHER_NAMES: [&str; 5] = ["rain", "snow", "sandstorm", "sporestorm", "fog"];

    if logic_team_from_name(name).is_some()
        || UNIT_NAMES.contains(&name)
        || WEATHER_NAMES.contains(&name)
    {
        return true;
    }

    let catalog = CATALOG.get_or_init(ContentCatalog::load_base_content);
    catalog.item_by_name(name).is_some()
        || catalog.liquid_by_name(name).is_some()
        || catalog.status_effect_by_name(name).is_some()
        || catalog.blocks.get_by_name(name).is_some()
}

pub fn logic_access_from_object_name(name: &str) -> Option<LAccess> {
    name.strip_prefix('@').and_then(LAccess::by_wire_name)
}

pub fn logic_content_name_from_object_name(name: &str) -> Option<&str> {
    name.strip_prefix('@')
}

pub fn lookup_logic_content_name(type_: ContentType, id: i32) -> Option<&'static str> {
    if id < 0 {
        return None;
    }

    let id = id as i16;
    let catalog = ContentCatalog::load_base_content();
    let name = match type_ {
        ContentType::Item => catalog
            .item_by_id(id)
            .map(|item| item.base.mappable.name.clone()),
        ContentType::Block => catalog
            .blocks
            .get(id)
            .map(|block| block.base().name.clone()),
        ContentType::Liquid => catalog
            .liquid_by_id(id)
            .map(|liquid| liquid.base.mappable.name.clone()),
        ContentType::Status => catalog
            .status_effect_by_id(id)
            .map(|status| status.base.mappable.name.clone()),
        _ => None,
    }?;
    Some(Box::leak(name.into_boxed_str()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalVarEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub privileged: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVarSnapshot {
    pub entries: Vec<GlobalVarEntry>,
}

impl GlobalVarSnapshot {
    pub fn baseline() -> Self {
        let mut entries = Vec::new();
        for name in [
            "sectionProcessor",
            "@this",
            "@thisx",
            "@thisy",
            "@links",
            "@ipt",
            "sectionGeneral",
            "false",
            "true",
            "@pi",
            "@e",
            "@degToRad",
            "@radToDeg",
            "sectionMap",
            "@time",
            "@tick",
            "@second",
            "@minute",
            "@waveNumber",
            "@waveTime",
            "@mapw",
            "@maph",
            "sectionNetwork",
            "@server",
            "@client",
            "@clientLocale",
            "@clientUnit",
            "@clientName",
            "@clientTeam",
            "@clientMobile",
            "sectionLookup",
        ] {
            let privileged = matches!(
                name,
                "@server"
                    | "@client"
                    | "@clientLocale"
                    | "@clientUnit"
                    | "@clientName"
                    | "@clientTeam"
                    | "@clientMobile"
            );
            entries.push(GlobalVarEntry {
                name,
                description: "",
                icon: "",
                privileged,
            });
        }
        Self { entries }
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.entries.iter().map(|entry| entry.name).collect()
    }

    pub fn visible_to_privileged(&self, privileged: bool) -> Vec<&'static str> {
        self.entries
            .iter()
            .filter(|entry| privileged || !entry.privileged)
            .map(|entry| entry.name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_vars_baseline_keeps_java_constants_and_entry_order() {
        assert_eq!(LOGIC_CTRL_PROCESSOR, 1);
        assert_eq!(LOGIC_CTRL_PLAYER, 2);
        assert_eq!(LOGIC_CTRL_COMMAND, 3);
        assert_eq!(
            LOOKABLE_CONTENT,
            ["block", "unit", "item", "liquid", "team"]
        );
        assert_eq!(
            WRITABLE_LOOKABLE_CONTENT,
            ["block", "unit", "item", "liquid"]
        );
        assert_eq!(
            LOOKABLE_CONTENT_TYPES,
            [
                ContentType::Block,
                ContentType::Unit,
                ContentType::Item,
                ContentType::Liquid,
                ContentType::Team
            ]
        );

        let snapshot = GlobalVarSnapshot::baseline();
        let names = snapshot.names();
        assert_eq!(
            &names[..13],
            [
                "sectionProcessor",
                "@this",
                "@thisx",
                "@thisy",
                "@links",
                "@ipt",
                "sectionGeneral",
                "false",
                "true",
                "@pi",
                "@e",
                "@degToRad",
                "@radToDeg"
            ]
        );
        assert!(names.contains(&"sectionMap"));
        assert!(names.contains(&"sectionNetwork"));
        assert_eq!(names.last(), Some(&"sectionLookup"));

        let public_names = snapshot.visible_to_privileged(false);
        assert!(!public_names.contains(&"@clientLocale"));
        assert!(!public_names.contains(&"@clientUnit"));
        assert!(snapshot
            .visible_to_privileged(true)
            .contains(&"@clientLocale"));
        assert!(
            snapshot
                .entries
                .iter()
                .find(|entry| entry.name == "@server")
                .unwrap()
                .privileged
        );
    }

    #[test]
    fn global_vars_resolve_java_constant_values_and_privileged_fallbacks() {
        assert_eq!(
            logic_global_value("false", false),
            Some(LogicValue::Number(0.0))
        );
        assert_eq!(
            logic_global_value("@ctrlProcessor", false),
            Some(LogicValue::Number(1.0))
        );
        assert_eq!(
            logic_global_value("@clientName", false),
            Some(LogicValue::Object(None))
        );
        assert_eq!(
            logic_global_value("@clientName", true),
            Some(LogicValue::Object(None))
        );
        assert!(matches!(
            logic_global_value("@colorScarlet", false),
            Some(LogicValue::Number(value)) if value.to_bits() == 0xff341cff
        ));
        assert_eq!(
            logic_global_value("@totalItems", false),
            Some(LogicValue::Object(Some("@totalItems".into())))
        );
        assert_eq!(
            logic_global_value("@sharded", false),
            Some(LogicValue::Object(Some("@sharded".into())))
        );
    }

    #[test]
    fn global_vars_lookup_helpers_match_logic_object_names() {
        assert_eq!(
            logic_access_from_object_name("@health"),
            Some(LAccess::Health)
        );
        assert_eq!(logic_access_from_object_name("health"), None);
        assert_eq!(
            logic_content_name_from_object_name("@copper"),
            Some("copper")
        );
        assert_eq!(logic_content_name_from_object_name("copper"), None);
        assert_eq!(
            lookup_logic_content_name(ContentType::Item, 0),
            Some("copper")
        );
        assert_eq!(lookup_logic_content_name(ContentType::Item, -1), None);
    }
}
