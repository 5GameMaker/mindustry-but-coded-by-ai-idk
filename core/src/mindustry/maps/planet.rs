//! Planet map generation data mirrors from upstream `mindustry.maps.planet`.

#[derive(Debug, Clone, PartialEq)]
pub struct AsteroidGenerator {
    pub min: i32,
    pub max: i32,
    pub octaves: i32,
    pub foct: i32,
    pub rad_min: f32,
    pub rad_max: f32,
    pub persistence: f32,
    pub scale: f32,
    pub mag: f32,
    pub thresh: f32,
    pub fmag: f32,
    pub fscl: f32,
    pub fper: f32,
    pub stone_chance: f32,
    pub ice_chance: f32,
    pub carbon_chance: f32,
    pub beryl_chance: f32,
    pub ferric_chance: f32,
    pub thorium_scl: f32,
    pub copper_scale: f32,
    pub lead_scale: f32,
    pub graphite_scale: f32,
    pub beryllium_scale: f32,
}

impl Default for AsteroidGenerator {
    fn default() -> Self {
        Self {
            min: 20,
            max: 30,
            octaves: 2,
            foct: 3,
            rad_min: 12.0,
            rad_max: 60.0,
            persistence: 0.4,
            scale: 30.0,
            mag: 0.46,
            thresh: 1.0,
            fmag: 0.5,
            fscl: 50.0,
            fper: 0.6,
            stone_chance: 0.0,
            ice_chance: 0.0,
            carbon_chance: 0.0,
            beryl_chance: 0.0,
            ferric_chance: 1.0,
            thorium_scl: 1.0,
            copper_scale: 1.0,
            lead_scale: 1.0,
            graphite_scale: 1.0,
            beryllium_scale: 1.0,
        }
    }
}

impl AsteroidGenerator {
    pub fn sector_size_hint(&self) -> i32 {
        self.min
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErekirPlanetGenerator {
    pub height_scl: f32,
    pub octaves: f32,
    pub persistence: f32,
    pub height_pow: f32,
    pub height_mult: f32,
}

impl ErekirPlanetGenerator {
    pub const ARK_THRESH: f32 = 0.28;
    pub const ARK_SCL: f32 = 0.83;
    pub const ARK_SEED: i32 = 7;
    pub const ARK_OCT: i32 = 2;
    pub const LIQ_THRESH: f32 = 0.64;
    pub const LIQ_SCL: f32 = 87.0;
    pub const RED_THRESH: f32 = 3.1;
    pub const NO_ARK_THRESH: f32 = 0.3;
    pub const CRYSTAL_SEED: i32 = 8;
    pub const CRYSTAL_OCT: i32 = 2;
    pub const CRYSTAL_SCL: f32 = 0.9;
    pub const CRYSTAL_MAG: f32 = 0.3;
    pub const AIR_THRESH: f32 = 0.13;
    pub const AIR_SCL: f32 = 14.0;
}

impl Default for ErekirPlanetGenerator {
    fn default() -> Self {
        Self {
            height_scl: 0.9,
            octaves: 8.0,
            persistence: 0.7,
            height_pow: 3.0,
            height_mult: 1.6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SerpuloPlanetGenerator {
    pub indirect_paths: bool,
    pub gen_lakes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TantrosPlanetGenerator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asteroid_defaults_match_upstream_fields() {
        let generator = AsteroidGenerator::default();
        assert_eq!(generator.min, 20);
        assert_eq!(generator.max, 30);
        assert_eq!(generator.octaves, 2);
        assert_eq!(generator.foct, 3);
        assert_eq!(generator.rad_min, 12.0);
        assert_eq!(generator.rad_max, 60.0);
        assert_eq!(generator.ferric_chance, 1.0);
    }

    #[test]
    fn planet_generator_constants_and_defaults_match_upstream_setup() {
        let erekir = ErekirPlanetGenerator::default();
        assert_eq!(erekir.height_scl, 0.9);
        assert_eq!(erekir.octaves, 8.0);
        assert_eq!(ErekirPlanetGenerator::ARK_THRESH, 0.28);
        assert_eq!(ErekirPlanetGenerator::CRYSTAL_SEED, 8);

        let serpulo = SerpuloPlanetGenerator::default();
        assert!(!serpulo.indirect_paths);
        assert!(!serpulo.gen_lakes);
        assert_eq!(
            format!("{:?}", TantrosPlanetGenerator::default()),
            "TantrosPlanetGenerator"
        );
    }
}
