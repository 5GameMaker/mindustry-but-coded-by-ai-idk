//! Heat-producing block interface mirroring upstream `mindustry.world.blocks.heat.HeatBlock`.

use super::HeatBlockState;

pub trait HeatBlock {
    fn heat(&self) -> f32;

    /// Heat as a fraction of max heat.
    fn heat_frac(&self) -> f32;
}

impl HeatBlock for HeatBlockState {
    fn heat(&self) -> f32 {
        self.heat
    }

    fn heat_frac(&self) -> f32 {
        HeatBlockState::heat_frac(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_block_state_implements_upstream_heat_block_contract() {
        let state = HeatBlockState {
            heat: 9.0,
            visual_max_heat: 18.0,
            split_heat: false,
        };

        assert_eq!(HeatBlock::heat(&state), 9.0);
        assert_eq!(HeatBlock::heat_frac(&state), 0.5);
    }
}
