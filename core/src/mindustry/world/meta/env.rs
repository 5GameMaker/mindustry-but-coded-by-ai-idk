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

#[cfg(test)]
mod tests {
    use super::Env;

    #[test]
    fn env_flags_match_java_bit_layout() {
        assert_eq!(Env::TERRESTRIAL, 1);
        assert_eq!(Env::SPACE, 1 << 1);
        assert_eq!(Env::UNDERWATER, 1 << 2);
        assert_eq!(Env::SPORES, 1 << 3);
        assert_eq!(Env::SCORCHING, 1 << 4);
        assert_eq!(Env::GROUND_OIL, 1 << 5);
        assert_eq!(Env::GROUND_WATER, 1 << 6);
        assert_eq!(Env::OXYGEN, 1 << 7);
        assert_eq!(Env::ANY, 0xffff_ffff);
        assert_eq!(Env::NONE, 0);
    }
}
