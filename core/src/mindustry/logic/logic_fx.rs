//! Mirrors upstream `mindustry.logic.LogicFx`.

#[derive(Debug, Clone, PartialEq)]
pub struct LogicEffectEntry {
    pub name: String,
    pub effect: &'static str,
    pub size: bool,
    pub rotate: bool,
    pub color: bool,
    pub data: Option<&'static str>,
    pub bounds: f32,
}

impl LogicEffectEntry {
    pub fn new(name: impl Into<String>, effect: &'static str) -> Self {
        Self {
            name: name.into(),
            effect,
            size: false,
            rotate: false,
            color: false,
            data: None,
            bounds: -1.0,
        }
    }

    pub fn size(mut self) -> Self {
        self.size = true;
        self
    }

    pub fn rotate(mut self) -> Self {
        self.rotate = true;
        self
    }

    pub fn color(mut self) -> Self {
        self.color = true;
        self
    }

    pub fn data(mut self, data: &'static str) -> Self {
        self.data = Some(data);
        self
    }

    pub fn bounds(mut self, bounds: f32) -> Self {
        self.bounds = bounds;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicEffectSpec {
    pub name: &'static str,
    pub effect: &'static str,
    pub size: bool,
    pub rotate: bool,
    pub color: bool,
    pub data: Option<&'static str>,
    pub bounds: f32,
}

impl LogicEffectSpec {
    pub fn to_entry(&self) -> LogicEffectEntry {
        LogicEffectEntry {
            name: self.name.to_string(),
            effect: self.effect,
            size: self.size,
            rotate: self.rotate,
            color: self.color,
            data: self.data,
            bounds: self.bounds,
        }
    }
}

pub const LOGIC_EFFECTS: [LogicEffectSpec; 33] = [
    LogicEffectSpec {
        name: "warn",
        effect: "unitCapKill",
        size: false,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "cross",
        effect: "unitEnvKill",
        size: false,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "blockFall",
        effect: "blockCrash",
        size: false,
        rotate: false,
        color: false,
        data: Some("Block"),
        bounds: 100.0,
    },
    LogicEffectSpec {
        name: "placeBlock",
        effect: "placeBlock",
        size: true,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "placeBlockSpark",
        effect: "coreLaunchConstruct",
        size: true,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "breakBlock",
        effect: "breakBlock",
        size: true,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "spawn",
        effect: "spawn",
        size: false,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "trail",
        effect: "colorTrail",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "breakProp",
        effect: "breakProp",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeCloud",
        effect: "missileTrailSmoke",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "vapor",
        effect: "vapor",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "hit",
        effect: "hitBulletColor",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "hitSquare",
        effect: "hitSquaresColor",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "shootSmall",
        effect: "shootSmall",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "shootBig",
        effect: "shootTitan",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeSmall",
        effect: "shootSmallSmoke",
        size: false,
        rotate: true,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeBig",
        effect: "shootBigSmoke",
        size: false,
        rotate: true,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeColor",
        effect: "shootSmokeTitan",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeSquare",
        effect: "shootSmokeSquare",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokeSquareBig",
        effect: "shootSmokeSquareBig",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "spark",
        effect: "hitLaserBlast",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "sparkBig",
        effect: "circleColorSpark",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "sparkShoot",
        effect: "colorSpark",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "sparkShootBig",
        effect: "randLifeSpark",
        size: false,
        rotate: true,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "drill",
        effect: "mine",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "drillBig",
        effect: "mineHuge",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "lightBlock",
        effect: "lightBlock",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "explosion",
        effect: "dynamicExplosion",
        size: true,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "smokePuff",
        effect: "smokePuff",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "sparkExplosion",
        effect: "titanExplosion",
        size: false,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "crossExplosion",
        effect: "dynamicSpikes",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "wave",
        effect: "dynamicWave",
        size: true,
        rotate: false,
        color: true,
        data: None,
        bounds: -1.0,
    },
    LogicEffectSpec {
        name: "bubble",
        effect: "airBubble",
        size: false,
        rotate: false,
        color: false,
        data: None,
        bounds: -1.0,
    },
];

pub fn logic_effect_names() -> Vec<&'static str> {
    LOGIC_EFFECTS.iter().map(|entry| entry.name).collect()
}

pub fn get_logic_effect(name: &str) -> Option<&'static LogicEffectSpec> {
    LOGIC_EFFECTS.iter().find(|entry| entry.name == name)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicEffectRegistry {
    entries: Vec<LogicEffectEntry>,
}

impl Default for LogicEffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicEffectRegistry {
    pub fn new() -> Self {
        Self {
            entries: LOGIC_EFFECTS
                .iter()
                .map(LogicEffectSpec::to_entry)
                .collect(),
        }
    }

    pub fn entries(&self) -> &[LogicEffectEntry] {
        &self.entries
    }

    pub fn all(&self) -> Vec<&str> {
        self.entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<&LogicEffectEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    pub fn add(&mut self, name: impl Into<String>, mut entry: LogicEffectEntry) {
        let name = name.into();
        entry.name = name.clone();
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|existing| existing.name == name)
        {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        get_logic_effect, logic_effect_names, LogicEffectEntry, LogicEffectRegistry, LOGIC_EFFECTS,
    };

    #[test]
    fn logic_fx_registry_matches_java_order_flags_and_extension_semantics() {
        assert_eq!(LOGIC_EFFECTS.len(), 33);
        assert_eq!(
            logic_effect_names(),
            vec![
                "warn",
                "cross",
                "blockFall",
                "placeBlock",
                "placeBlockSpark",
                "breakBlock",
                "spawn",
                "trail",
                "breakProp",
                "smokeCloud",
                "vapor",
                "hit",
                "hitSquare",
                "shootSmall",
                "shootBig",
                "smokeSmall",
                "smokeBig",
                "smokeColor",
                "smokeSquare",
                "smokeSquareBig",
                "spark",
                "sparkBig",
                "sparkShoot",
                "sparkShootBig",
                "drill",
                "drillBig",
                "lightBlock",
                "explosion",
                "smokePuff",
                "sparkExplosion",
                "crossExplosion",
                "wave",
                "bubble"
            ]
        );

        let block_fall = get_logic_effect("blockFall").unwrap();
        assert_eq!(block_fall.effect, "blockCrash");
        assert_eq!(block_fall.data, Some("Block"));
        assert_eq!(block_fall.bounds, 100.0);
        assert!(!block_fall.size);
        assert!(!block_fall.rotate);
        assert!(!block_fall.color);

        let trail = get_logic_effect("trail").unwrap();
        assert_eq!(trail.effect, "colorTrail");
        assert!(trail.size);
        assert!(trail.color);
        assert!(!trail.rotate);

        let shoot_big = get_logic_effect("shootBig").unwrap();
        assert_eq!(shoot_big.effect, "shootTitan");
        assert!(shoot_big.rotate);
        assert!(shoot_big.color);
        assert!(!shoot_big.size);

        let wave = get_logic_effect("wave").unwrap();
        assert_eq!(wave.effect, "dynamicWave");
        assert!(wave.size);
        assert!(wave.color);

        assert_eq!(get_logic_effect("missing"), None);

        let mut registry = LogicEffectRegistry::new();
        assert_eq!(registry.all().first(), Some(&"warn"));
        assert_eq!(registry.all().last(), Some(&"bubble"));
        registry.add(
            "custom",
            LogicEffectEntry::new("ignored", "customFx")
                .size()
                .rotate()
                .color()
                .bounds(42.0),
        );
        let custom = registry.get("custom").unwrap();
        assert_eq!(custom.name, "custom");
        assert_eq!(custom.effect, "customFx");
        assert!(custom.size && custom.rotate && custom.color);
        assert_eq!(custom.bounds, 42.0);
        assert_eq!(registry.all().last(), Some(&"custom"));

        registry.add("warn", LogicEffectEntry::new("ignored", "replacement"));
        assert_eq!(registry.get("warn").unwrap().name, "warn");
        assert_eq!(registry.get("warn").unwrap().effect, "replacement");
        assert_eq!(registry.all().first(), Some(&"warn"));
    }
}
