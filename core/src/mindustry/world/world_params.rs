#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldParams {
    pub seed_offset: i32,
    pub save_info: bool,
    pub core_position_override: i32,
}

impl WorldParams {
    pub fn new() -> Self {
        Self {
            seed_offset: 0,
            save_info: true,
            core_position_override: 0,
        }
    }
}

impl Default for WorldParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_params_defaults_match_java_field_initializers() {
        let params = WorldParams::default();

        assert_eq!(params.seed_offset, 0);
        assert!(params.save_info);
        assert_eq!(params.core_position_override, 0);
        assert_eq!(WorldParams::new(), params);
    }
}
