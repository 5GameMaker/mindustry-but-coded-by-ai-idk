/// Sorting layers used for rendering. Values mirror upstream
/// `mindustry.graphics.Layer` and should stay in ascending order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerEntry {
    pub name: &'static str,
    pub value: f32,
}

impl LayerEntry {
    pub const fn new(name: &'static str, value: f32) -> Self {
        Self { name, value }
    }
}

/// Stores constants for sorting layers.
pub struct Layer;

impl Layer {
    // min layer
    pub const MIN: f32 = -11.0;

    // background, which may be planets or an image or nothing at all
    pub const BACKGROUND: f32 = -10.0;

    // floor tiles
    pub const FLOOR: f32 = 0.0;

    // scorch marks on the floor
    pub const SCORCH: f32 = 10.0;

    // things such as spent casings or rubble
    pub const DEBRIS: f32 = 20.0;

    // stuff under blocks, like connections of conveyors/conduits
    pub const BLOCK_UNDER: f32 = 29.5;

    // base block layer - most blocks go here
    pub const BLOCK: f32 = 30.0;

    // layer for cracks over blocks, batched to prevent excessive texture swaps
    pub const BLOCK_CRACKS: f32 = 30.0_f32 + 0.1_f32;

    // some blocks need to draw stuff after cracks
    pub const BLOCK_AFTER_CRACKS: f32 = 30.0_f32 + 0.2_f32;

    // informal layer used for additive blending overlay, grouped together to reduce draw calls
    pub const BLOCK_ADDITIVE: f32 = 31.0;

    // props such as boulders
    pub const BLOCK_PROP: f32 = 32.0;

    // things drawn over blocks (intermediate layer)
    pub const BLOCK_OVER: f32 = 35.0;

    // blocks currently in progress *shaders used*
    pub const BLOCK_BUILDING: f32 = 40.0;

    // turrets
    pub const TURRET: f32 = 50.0;

    // special layer for turret additive blending heat stuff
    pub const TURRET_HEAT: f32 = 50.0_f32 + 0.1_f32;

    // ground units
    pub const GROUND_UNIT: f32 = 60.0;

    // power lines
    pub const POWER: f32 = 70.0;

    // certain multi-legged units
    pub const LEG_UNIT: f32 = 75.0;

    // darkness over block clusters
    pub const DARKNESS: f32 = 80.0;

    // building plans
    pub const PLANS: f32 = 85.0;

    // flying units (low altitude)
    pub const FLYING_UNIT_LOW: f32 = 90.0;

    // bullets *bloom begin*
    pub const BULLET: f32 = 100.0;

    // effects *bloom end*
    pub const EFFECT: f32 = 110.0;

    // flying units
    pub const FLYING_UNIT: f32 = 115.0;

    // overlaid UI, like block config guides
    pub const OVERLAY_UI: f32 = 120.0;

    // build beam effects
    pub const BUILD_BEAM: f32 = 122.0;

    // shield effects
    pub const SHIELDS: f32 = 125.0;

    // weather effects, e.g. rain and snow
    pub const WEATHER: f32 = 130.0;

    // light rendering *shaders used*
    pub const LIGHT: f32 = 140.0;

    // names of players in the game
    pub const PLAYER_NAME: f32 = 150.0;

    // fog of war effect, if applicable
    pub const FOG_OF_WAR: f32 = 155.0;

    // space effects, currently only the land and launch effects
    pub const SPACE: f32 = 160.0;

    // the end of all layers
    pub const END: f32 = 200.0;

    // things after pixelation - used for text
    pub const END_PIXELED: f32 = 210.0;

    // max layer
    pub const MAX: f32 = 220.0;

    pub const ALL: [LayerEntry; 35] = [
        LayerEntry::new("min", Self::MIN),
        LayerEntry::new("background", Self::BACKGROUND),
        LayerEntry::new("floor", Self::FLOOR),
        LayerEntry::new("scorch", Self::SCORCH),
        LayerEntry::new("debris", Self::DEBRIS),
        LayerEntry::new("blockUnder", Self::BLOCK_UNDER),
        LayerEntry::new("block", Self::BLOCK),
        LayerEntry::new("blockCracks", Self::BLOCK_CRACKS),
        LayerEntry::new("blockAfterCracks", Self::BLOCK_AFTER_CRACKS),
        LayerEntry::new("blockAdditive", Self::BLOCK_ADDITIVE),
        LayerEntry::new("blockProp", Self::BLOCK_PROP),
        LayerEntry::new("blockOver", Self::BLOCK_OVER),
        LayerEntry::new("blockBuilding", Self::BLOCK_BUILDING),
        LayerEntry::new("turret", Self::TURRET),
        LayerEntry::new("turretHeat", Self::TURRET_HEAT),
        LayerEntry::new("groundUnit", Self::GROUND_UNIT),
        LayerEntry::new("power", Self::POWER),
        LayerEntry::new("legUnit", Self::LEG_UNIT),
        LayerEntry::new("darkness", Self::DARKNESS),
        LayerEntry::new("plans", Self::PLANS),
        LayerEntry::new("flyingUnitLow", Self::FLYING_UNIT_LOW),
        LayerEntry::new("bullet", Self::BULLET),
        LayerEntry::new("effect", Self::EFFECT),
        LayerEntry::new("flyingUnit", Self::FLYING_UNIT),
        LayerEntry::new("overlayUI", Self::OVERLAY_UI),
        LayerEntry::new("buildBeam", Self::BUILD_BEAM),
        LayerEntry::new("shields", Self::SHIELDS),
        LayerEntry::new("weather", Self::WEATHER),
        LayerEntry::new("light", Self::LIGHT),
        LayerEntry::new("playerName", Self::PLAYER_NAME),
        LayerEntry::new("fogOfWar", Self::FOG_OF_WAR),
        LayerEntry::new("space", Self::SPACE),
        LayerEntry::new("end", Self::END),
        LayerEntry::new("endPixeled", Self::END_PIXELED),
        LayerEntry::new("max", Self::MAX),
    ];

    pub fn ordered_entries() -> &'static [LayerEntry; 35] {
        &Self::ALL
    }

    pub fn by_name(name: &str) -> Option<f32> {
        Self::ordered_entries()
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.value)
    }
}

#[cfg(test)]
mod tests {
    use super::Layer;

    #[test]
    fn layer_constants_match_java_values() {
        assert_eq!(Layer::ALL.len(), 35);
        assert_eq!(Layer::MIN, -11.0);
        assert_eq!(Layer::BACKGROUND, -10.0);
        assert_eq!(Layer::FLOOR, 0.0);
        assert_eq!(Layer::SCORCH, 10.0);
        assert_eq!(Layer::DEBRIS, 20.0);
        assert_eq!(Layer::BLOCK_UNDER, 29.5);
        assert_eq!(Layer::BLOCK, 30.0);
        assert_eq!(
            Layer::BLOCK_CRACKS.to_bits(),
            (30.0_f32 + 0.1_f32).to_bits()
        );
        assert_eq!(
            Layer::BLOCK_AFTER_CRACKS.to_bits(),
            (30.0_f32 + 0.2_f32).to_bits()
        );
        assert_eq!(Layer::BLOCK_ADDITIVE, 31.0);
        assert_eq!(Layer::BLOCK_PROP, 32.0);
        assert_eq!(Layer::BLOCK_OVER, 35.0);
        assert_eq!(Layer::BLOCK_BUILDING, 40.0);
        assert_eq!(Layer::TURRET, 50.0);
        assert_eq!(Layer::TURRET_HEAT.to_bits(), (50.0_f32 + 0.1_f32).to_bits());
        assert_eq!(Layer::GROUND_UNIT, 60.0);
        assert_eq!(Layer::POWER, 70.0);
        assert_eq!(Layer::LEG_UNIT, 75.0);
        assert_eq!(Layer::DARKNESS, 80.0);
        assert_eq!(Layer::PLANS, 85.0);
        assert_eq!(Layer::FLYING_UNIT_LOW, 90.0);
        assert_eq!(Layer::BULLET, 100.0);
        assert_eq!(Layer::EFFECT, 110.0);
        assert_eq!(Layer::FLYING_UNIT, 115.0);
        assert_eq!(Layer::OVERLAY_UI, 120.0);
        assert_eq!(Layer::BUILD_BEAM, 122.0);
        assert_eq!(Layer::SHIELDS, 125.0);
        assert_eq!(Layer::WEATHER, 130.0);
        assert_eq!(Layer::LIGHT, 140.0);
        assert_eq!(Layer::PLAYER_NAME, 150.0);
        assert_eq!(Layer::FOG_OF_WAR, 155.0);
        assert_eq!(Layer::SPACE, 160.0);
        assert_eq!(Layer::END, 200.0);
        assert_eq!(Layer::END_PIXELED, 210.0);
        assert_eq!(Layer::MAX, 220.0);
    }

    #[test]
    fn layer_ordered_entries_match_upstream_sequence() {
        let expected_names = [
            "min",
            "background",
            "floor",
            "scorch",
            "debris",
            "blockUnder",
            "block",
            "blockCracks",
            "blockAfterCracks",
            "blockAdditive",
            "blockProp",
            "blockOver",
            "blockBuilding",
            "turret",
            "turretHeat",
            "groundUnit",
            "power",
            "legUnit",
            "darkness",
            "plans",
            "flyingUnitLow",
            "bullet",
            "effect",
            "flyingUnit",
            "overlayUI",
            "buildBeam",
            "shields",
            "weather",
            "light",
            "playerName",
            "fogOfWar",
            "space",
            "end",
            "endPixeled",
            "max",
        ];

        assert_eq!(Layer::ordered_entries(), &Layer::ALL);
        for (entry, expected_name) in Layer::ordered_entries().iter().zip(expected_names) {
            assert_eq!(entry.name, expected_name);
        }

        for window in Layer::ordered_entries().windows(2) {
            assert!(window[0].value < window[1].value);
        }
    }

    #[test]
    fn layer_lookup_by_name_works() {
        assert_eq!(Layer::by_name("blockCracks"), Some(Layer::BLOCK_CRACKS));
        assert_eq!(Layer::by_name("turretHeat"), Some(Layer::TURRET_HEAT));
        assert_eq!(Layer::by_name("missing"), None);
    }
}
