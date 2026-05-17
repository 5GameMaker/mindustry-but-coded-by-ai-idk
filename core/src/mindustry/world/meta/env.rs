pub struct Env;

impl Env {
    pub const TERRESTRIAL: u32 = 1;
    pub const SPACE: u32 = 1 << 1;
    pub const UNDERWATER: u32 = 1 << 2;
    pub const SPORES: u32 = 1 << 3;
    pub const SCORCHING: u32 = 1 << 4;
    pub const GROUND_OIL: u32 = 1 << 5;
    pub const GROUND_WATER: u32 = 1 << 6;
    pub const OXYGEN: u32 = 1 << 7;
    pub const ANY: u32 = 0xffff_ffff;
    pub const NONE: u32 = 0;
}
