use crate::mindustry::entities::comp::DecalColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PalEntry {
    pub name: &'static str,
    pub color: DecalColor,
}

impl PalEntry {
    pub const fn new(name: &'static str, color: DecalColor) -> Self {
        Self { name, color }
    }
}

pub struct Pal;

impl Pal {
    pub const WATER: DecalColor = DecalColor::from_rgba(0x596ab8ff);
    pub const DARK_OUTLINE: DecalColor = DecalColor::from_rgba(0x2d2f39ff);
    pub const THORIUM_PINK: DecalColor = DecalColor::from_rgba(0xf9a3c7ff);
    pub const COAL_BLACK: DecalColor = DecalColor::from_rgba(0x272727ff);
    pub const ITEMS: DecalColor = DecalColor::from_rgba(0x2ea756ff);
    pub const COMMAND: DecalColor = DecalColor::from_rgba(0xeab678ff);
    pub const SAP: DecalColor = DecalColor::from_rgba(0x665c9fff);
    pub const SAP_BULLET: DecalColor = DecalColor::from_rgba(0xbf92f9ff);
    pub const SAP_BULLET_BACK: DecalColor = DecalColor::from_rgba(0x6d56bfff);
    pub const SUPPRESS: DecalColor = Self::SAP.mul(1.6);
    pub const REGEN: DecalColor = DecalColor::from_rgba(0xd1efffff);
    pub const REACTOR_PURPLE: DecalColor = DecalColor::from_rgba(0xbf92f9ff);
    pub const REACTOR_PURPLE2: DecalColor = DecalColor::from_rgba(0x8a73c6ff);
    pub const SPORE: DecalColor = DecalColor::from_rgba(0x7457ceff);
    pub const SHIELD: DecalColor = DecalColor::from_rgba(0xffd37fff).with_alpha(0.7);
    pub const BULLET_YELLOW: DecalColor = DecalColor::from_rgba(0xfff8e8ff);
    pub const BULLET_YELLOW_BACK: DecalColor = DecalColor::from_rgba(0xf9c27aff);
    pub const DARK_METAL: DecalColor = DecalColor::from_rgba(0x6e7080ff);
    pub const DARKER_METAL: DecalColor = DecalColor::from_rgba(0x565666ff);
    pub const DARKEST_METAL: DecalColor = DecalColor::from_rgba(0x38393fff);
    pub const MISSILE_YELLOW: DecalColor = DecalColor::from_rgba(0xffd2aeff);
    pub const MISSILE_YELLOW_BACK: DecalColor = DecalColor::from_rgba(0xe58956ff);
    pub const MELTDOWN_HIT: DecalColor = DecalColor::from_rgba(0xffb98bff);
    pub const PLASTANIUM_BACK: DecalColor = DecalColor::from_rgba(0xd8d97fff);
    pub const PLASTANIUM_FRONT: DecalColor = DecalColor::from_rgba(0xfffac6ff);
    pub const LIGHT_FLAME: DecalColor = DecalColor::from_rgba(0xffdd55ff);
    pub const DARK_FLAME: DecalColor = DecalColor::from_rgba(0xdb401cff);
    pub const LIGHT_PYRA_FLAME: DecalColor = DecalColor::from_rgba(0xffb855ff);
    pub const DARK_PYRA_FLAME: DecalColor = DecalColor::from_rgba(0xdb661cff);
    pub const TURRET_HEAT: DecalColor = DecalColor::from_rgba(0xab3400ff);
    pub const LIGHT_ORANGE: DecalColor = DecalColor::from_rgba(0xf68021ff);
    pub const LIGHTISH_ORANGE: DecalColor = DecalColor::from_rgba(0xf8ad42ff);
    pub const LIGHTER_ORANGE: DecalColor = DecalColor::from_rgba(0xf6e096ff);
    pub const LIGHTISH_GRAY: DecalColor = DecalColor::from_rgba(0xa2a2a2ff);
    pub const DARKISH_GRAY: DecalColor = DecalColor {
        r: 0.3,
        g: 0.3,
        b: 0.3,
        a: 1.0,
    };
    pub const DARKER_GRAY: DecalColor = DecalColor {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    };
    pub const DARKEST_GRAY: DecalColor = DecalColor {
        r: 0.1,
        g: 0.1,
        b: 0.1,
        a: 1.0,
    };
    pub const DARKESTEST_GRAY: DecalColor = DecalColor {
        r: 0.05,
        g: 0.05,
        b: 0.05,
        a: 1.0,
    };
    pub const SHADOW: DecalColor = DecalColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.22,
    };
    pub const AMMO: DecalColor = DecalColor::from_rgba(0xff8947ff);
    pub const RUBBLE: DecalColor = DecalColor::from_rgba(0x1c1817ff);
    pub const BOOST_TO: DecalColor = DecalColor::from_rgba(0xffad4dff);
    pub const BOOST_FROM: DecalColor = DecalColor::from_rgba(0xff7f57ff);
    pub const LANCER_LASER: DecalColor = DecalColor::from_rgba(0xa9d8ffff);
    pub const STONE_GRAY: DecalColor = DecalColor::from_rgba(0x8f8f8fff);
    pub const ENGINE: DecalColor = DecalColor::from_rgba(0xffbb64ff);
    pub const YELLOW_BOLT_FRONT: DecalColor = DecalColor::from_rgba(0xffd27eff);
    pub const HEALTH: DecalColor = DecalColor::from_rgba(0xff341cff);
    pub const HEAL: DecalColor = DecalColor::from_rgba(0x98ffa9ff);
    pub const BAR: DecalColor = DecalColor::from_rgba(0x708090ff);
    pub const ACCENT: DecalColor = DecalColor::from_rgba(0xffd37fff);
    pub const STAT: DecalColor = DecalColor::from_rgba(0xffd37fff);
    pub const NEGATIVE_STAT: DecalColor = DecalColor::from_rgba(0xe55454ff);
    pub const GRAY: DecalColor = DecalColor::from_rgba(0x454545ff);
    pub const METAL_GRAY_DARK: DecalColor = DecalColor::from_rgba(0x6e7080ff);
    pub const ACCENT_BACK: DecalColor = DecalColor::from_rgba(0xd4816bff);
    pub const PLACE: DecalColor = DecalColor::from_rgba(0x6335f8ff);
    pub const REMOVE: DecalColor = DecalColor::from_rgba(0xe55454ff);
    pub const NOPLACE: DecalColor = DecalColor::from_rgba(0xffa697ff);
    pub const REMOVE_BACK: DecalColor = DecalColor::from_rgba(0xa73e3eff);
    pub const PLACE_ROTATE: DecalColor = Self::ACCENT;
    pub const BREAK_INVALID: DecalColor = DecalColor::from_rgba(0xd44b3dff);
    pub const RANGE: DecalColor = DecalColor::from_rgba(0xf4ba6eff);
    pub const POWER: DecalColor = DecalColor::from_rgba(0xfbad67ff);
    pub const POWER_BAR: DecalColor = DecalColor::from_rgba(0xec7b4cff);
    pub const POWER_LIGHT: DecalColor = DecalColor::from_rgba(0xfbd367ff);
    pub const PLACING: DecalColor = Self::ACCENT;
    pub const UNIT_FRONT: DecalColor = DecalColor::from_rgba(0xffa665ff);
    pub const UNIT_BACK: DecalColor = DecalColor::from_rgba(0xd06b53ff);
    pub const LIGHT_TRAIL: DecalColor = DecalColor::from_rgba(0xffe2a9ff);
    pub const SURGE: DecalColor = DecalColor::from_rgba(0xf3e979ff);
    pub const PLASTANIUM: DecalColor = DecalColor::from_rgba(0xa1b46eff);
    pub const RED_SPARK: DecalColor = DecalColor::from_rgba(0xfbb97fff);
    pub const ORANGE_SPARK: DecalColor = DecalColor::from_rgba(0xd2b29cff);
    pub const RED_DUST: DecalColor = DecalColor::from_rgba(0xffa480ff);
    pub const REDDER_DUST: DecalColor = DecalColor::from_rgba(0xff7b69ff);
    pub const PLASTIC_SMOKE: DecalColor = DecalColor::from_rgba(0xf1e479ff);
    pub const ADMIN_CHAT: DecalColor = DecalColor::from_rgba(0xff4000ff);
    pub const NEOPLASM_OUTLINE: DecalColor = DecalColor::from_rgba(0x2e191dff);
    pub const NEOPLASM1: DecalColor = DecalColor::from_rgba(0xf98f4aff);
    pub const NEOPLASM_MID: DecalColor = DecalColor::from_rgba(0xe05438ff);
    pub const NEOPLASM2: DecalColor = DecalColor::from_rgba(0x9e172cff);
    pub const NEOPLASM_ACID: DecalColor = DecalColor::from_rgba(0x8ead44ff);
    pub const NEOPLASM_ACID_GLOW: DecalColor = DecalColor::from_rgba(0x68e43eff);
    pub const LOGIC_BLOCKS: DecalColor = DecalColor::from_rgba(0xd4816bff);
    pub const LOGIC_CONTROL: DecalColor = DecalColor::from_rgba(0x6bb2b2ff);
    pub const LOGIC_OPERATIONS: DecalColor = DecalColor::from_rgba(0x877badff);
    pub const LOGIC_IO: DecalColor = DecalColor::from_rgba(0xa08a8aff);
    pub const LOGIC_UNITS: DecalColor = DecalColor::from_rgba(0xc7b59dff);
    pub const LOGIC_WORLD: DecalColor = DecalColor::from_rgba(0x6b84d4ff);
    pub const BERYL_SHOT: DecalColor = DecalColor::from_rgba(0xb1dd7eff);
    pub const TUNGSTEN_SHOT: DecalColor = DecalColor::from_rgba(0x768a9aff);
    pub const PLASTIC_BURN: DecalColor = DecalColor::from_rgba(0xe9ead3ff);
    pub const MUDDY: DecalColor = DecalColor::from_rgba(0x432722ff);
    pub const RED_LIGHT: DecalColor = DecalColor::from_rgba(0xfeb380ff);
    pub const SLAG_ORANGE: DecalColor = DecalColor::from_rgba(0xffa166ff);
    pub const TECH_BLUE: DecalColor = DecalColor::from_rgba(0x8ca9e8ff);
    pub const VENT: DecalColor = DecalColor::from_rgba(0x6b4e4eff);
    pub const VENT2: DecalColor = DecalColor::from_rgba(0x3b2a2aff);
    pub const COPPER_AMMO_FRONT: DecalColor = DecalColor::from_rgba(0xeac1a8ff);
    pub const COPPER_AMMO_BACK: DecalColor = DecalColor::from_rgba(0xd39169ff);
    pub const GRAPHITE_AMMO_BACK: DecalColor = DecalColor::from_rgba(0x7d89d8ff);
    pub const GRAPHITE_AMMO_FRONT: DecalColor = DecalColor::from_rgba(0xdae1eeff);
    pub const SILICON_AMMO_BACK: DecalColor = DecalColor::from_rgba(0x707594ff);
    pub const SILICON_AMMO_FRONT: DecalColor = DecalColor::from_rgba(0x999ba0ff);
    pub const GLASS_AMMO_BACK: DecalColor = DecalColor::from_rgba(0xb9c9dfff);
    pub const GLASS_AMMO_FRONT: DecalColor = DecalColor::from_rgba(0xffffffff);
    pub const SCRAP_AMMO_FRONT: DecalColor = DecalColor::from_rgba(0xf5e0ccff);
    pub const SCRAP_AMMO_BACK: DecalColor = DecalColor::from_rgba(0xd8887eff);
    pub const SURGE_AMMO_FRONT: DecalColor = DecalColor::WHITE;
    pub const SURGE_AMMO_BACK: DecalColor = Self::SURGE;
    pub const BLAST_AMMO_BACK: DecalColor = DecalColor::from_rgba(0xe9665bff);
    pub const BLAST_AMMO_FRONT: DecalColor = DecalColor::from_rgba(0xeeab89ff);
    pub const THORIUM_AMMO_BACK: DecalColor = DecalColor::from_rgba(0xf595beff);
    pub const THORIUM_AMMO_FRONT: DecalColor = DecalColor::WHITE;

    pub const ALL: [PalEntry; 115] = [
        PalEntry::new("water", Self::WATER),
        PalEntry::new("darkOutline", Self::DARK_OUTLINE),
        PalEntry::new("thoriumPink", Self::THORIUM_PINK),
        PalEntry::new("coalBlack", Self::COAL_BLACK),
        PalEntry::new("items", Self::ITEMS),
        PalEntry::new("command", Self::COMMAND),
        PalEntry::new("sap", Self::SAP),
        PalEntry::new("sapBullet", Self::SAP_BULLET),
        PalEntry::new("sapBulletBack", Self::SAP_BULLET_BACK),
        PalEntry::new("suppress", Self::SUPPRESS),
        PalEntry::new("regen", Self::REGEN),
        PalEntry::new("reactorPurple", Self::REACTOR_PURPLE),
        PalEntry::new("reactorPurple2", Self::REACTOR_PURPLE2),
        PalEntry::new("spore", Self::SPORE),
        PalEntry::new("shield", Self::SHIELD),
        PalEntry::new("bulletYellow", Self::BULLET_YELLOW),
        PalEntry::new("bulletYellowBack", Self::BULLET_YELLOW_BACK),
        PalEntry::new("darkMetal", Self::DARK_METAL),
        PalEntry::new("darkerMetal", Self::DARKER_METAL),
        PalEntry::new("darkestMetal", Self::DARKEST_METAL),
        PalEntry::new("missileYellow", Self::MISSILE_YELLOW),
        PalEntry::new("missileYellowBack", Self::MISSILE_YELLOW_BACK),
        PalEntry::new("meltdownHit", Self::MELTDOWN_HIT),
        PalEntry::new("plastaniumBack", Self::PLASTANIUM_BACK),
        PalEntry::new("plastaniumFront", Self::PLASTANIUM_FRONT),
        PalEntry::new("lightFlame", Self::LIGHT_FLAME),
        PalEntry::new("darkFlame", Self::DARK_FLAME),
        PalEntry::new("lightPyraFlame", Self::LIGHT_PYRA_FLAME),
        PalEntry::new("darkPyraFlame", Self::DARK_PYRA_FLAME),
        PalEntry::new("turretHeat", Self::TURRET_HEAT),
        PalEntry::new("lightOrange", Self::LIGHT_ORANGE),
        PalEntry::new("lightishOrange", Self::LIGHTISH_ORANGE),
        PalEntry::new("lighterOrange", Self::LIGHTER_ORANGE),
        PalEntry::new("lightishGray", Self::LIGHTISH_GRAY),
        PalEntry::new("darkishGray", Self::DARKISH_GRAY),
        PalEntry::new("darkerGray", Self::DARKER_GRAY),
        PalEntry::new("darkestGray", Self::DARKEST_GRAY),
        PalEntry::new("darkestestGray", Self::DARKESTEST_GRAY),
        PalEntry::new("shadow", Self::SHADOW),
        PalEntry::new("ammo", Self::AMMO),
        PalEntry::new("rubble", Self::RUBBLE),
        PalEntry::new("boostTo", Self::BOOST_TO),
        PalEntry::new("boostFrom", Self::BOOST_FROM),
        PalEntry::new("lancerLaser", Self::LANCER_LASER),
        PalEntry::new("stoneGray", Self::STONE_GRAY),
        PalEntry::new("engine", Self::ENGINE),
        PalEntry::new("yellowBoltFront", Self::YELLOW_BOLT_FRONT),
        PalEntry::new("health", Self::HEALTH),
        PalEntry::new("heal", Self::HEAL),
        PalEntry::new("bar", Self::BAR),
        PalEntry::new("accent", Self::ACCENT),
        PalEntry::new("stat", Self::STAT),
        PalEntry::new("negativeStat", Self::NEGATIVE_STAT),
        PalEntry::new("gray", Self::GRAY),
        PalEntry::new("metalGrayDark", Self::METAL_GRAY_DARK),
        PalEntry::new("accentBack", Self::ACCENT_BACK),
        PalEntry::new("place", Self::PLACE),
        PalEntry::new("remove", Self::REMOVE),
        PalEntry::new("noplace", Self::NOPLACE),
        PalEntry::new("removeBack", Self::REMOVE_BACK),
        PalEntry::new("placeRotate", Self::PLACE_ROTATE),
        PalEntry::new("breakInvalid", Self::BREAK_INVALID),
        PalEntry::new("range", Self::RANGE),
        PalEntry::new("power", Self::POWER),
        PalEntry::new("powerBar", Self::POWER_BAR),
        PalEntry::new("powerLight", Self::POWER_LIGHT),
        PalEntry::new("placing", Self::PLACING),
        PalEntry::new("unitFront", Self::UNIT_FRONT),
        PalEntry::new("unitBack", Self::UNIT_BACK),
        PalEntry::new("lightTrail", Self::LIGHT_TRAIL),
        PalEntry::new("surge", Self::SURGE),
        PalEntry::new("plastanium", Self::PLASTANIUM),
        PalEntry::new("redSpark", Self::RED_SPARK),
        PalEntry::new("orangeSpark", Self::ORANGE_SPARK),
        PalEntry::new("redDust", Self::RED_DUST),
        PalEntry::new("redderDust", Self::REDDER_DUST),
        PalEntry::new("plasticSmoke", Self::PLASTIC_SMOKE),
        PalEntry::new("adminChat", Self::ADMIN_CHAT),
        PalEntry::new("neoplasmOutline", Self::NEOPLASM_OUTLINE),
        PalEntry::new("neoplasm1", Self::NEOPLASM1),
        PalEntry::new("neoplasmMid", Self::NEOPLASM_MID),
        PalEntry::new("neoplasm2", Self::NEOPLASM2),
        PalEntry::new("neoplasmAcid", Self::NEOPLASM_ACID),
        PalEntry::new("neoplasmAcidGlow", Self::NEOPLASM_ACID_GLOW),
        PalEntry::new("logicBlocks", Self::LOGIC_BLOCKS),
        PalEntry::new("logicControl", Self::LOGIC_CONTROL),
        PalEntry::new("logicOperations", Self::LOGIC_OPERATIONS),
        PalEntry::new("logicIo", Self::LOGIC_IO),
        PalEntry::new("logicUnits", Self::LOGIC_UNITS),
        PalEntry::new("logicWorld", Self::LOGIC_WORLD),
        PalEntry::new("berylShot", Self::BERYL_SHOT),
        PalEntry::new("tungstenShot", Self::TUNGSTEN_SHOT),
        PalEntry::new("plasticBurn", Self::PLASTIC_BURN),
        PalEntry::new("muddy", Self::MUDDY),
        PalEntry::new("redLight", Self::RED_LIGHT),
        PalEntry::new("slagOrange", Self::SLAG_ORANGE),
        PalEntry::new("techBlue", Self::TECH_BLUE),
        PalEntry::new("vent", Self::VENT),
        PalEntry::new("vent2", Self::VENT2),
        PalEntry::new("copperAmmoFront", Self::COPPER_AMMO_FRONT),
        PalEntry::new("copperAmmoBack", Self::COPPER_AMMO_BACK),
        PalEntry::new("graphiteAmmoBack", Self::GRAPHITE_AMMO_BACK),
        PalEntry::new("graphiteAmmoFront", Self::GRAPHITE_AMMO_FRONT),
        PalEntry::new("siliconAmmoBack", Self::SILICON_AMMO_BACK),
        PalEntry::new("siliconAmmoFront", Self::SILICON_AMMO_FRONT),
        PalEntry::new("glassAmmoBack", Self::GLASS_AMMO_BACK),
        PalEntry::new("glassAmmoFront", Self::GLASS_AMMO_FRONT),
        PalEntry::new("scrapAmmoFront", Self::SCRAP_AMMO_FRONT),
        PalEntry::new("scrapAmmoBack", Self::SCRAP_AMMO_BACK),
        PalEntry::new("surgeAmmoFront", Self::SURGE_AMMO_FRONT),
        PalEntry::new("surgeAmmoBack", Self::SURGE_AMMO_BACK),
        PalEntry::new("blastAmmoBack", Self::BLAST_AMMO_BACK),
        PalEntry::new("blastAmmoFront", Self::BLAST_AMMO_FRONT),
        PalEntry::new("thoriumAmmoBack", Self::THORIUM_AMMO_BACK),
        PalEntry::new("thoriumAmmoFront", Self::THORIUM_AMMO_FRONT),
    ];

    pub fn by_name(name: &str) -> Option<DecalColor> {
        Self::ALL
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.color)
    }
}
