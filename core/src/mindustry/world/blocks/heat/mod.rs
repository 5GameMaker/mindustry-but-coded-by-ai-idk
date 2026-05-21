use std::io::{self, Read, Write};

pub mod heat_block;
pub mod heat_consumer;

pub use heat_block::HeatBlock;
pub use heat_consumer::HeatConsumer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatBlockState {
    pub heat: f32,
    pub visual_max_heat: f32,
    pub split_heat: bool,
}

impl HeatBlockState {
    pub fn heat_frac(&self) -> f32 {
        heat_frac(self.heat, self.visual_max_heat, self.split_heat)
    }
}

pub fn heat_frac(heat: f32, visual_max_heat: f32, split_heat: bool) -> f32 {
    (heat / visual_max_heat) / if split_heat { 3.0 } else { 1.0 }
}

pub fn heat_consumer_requirement_met(heat: f32, heat_requirement: f32) -> bool {
    heat_requirement <= 0.0 || heat >= heat_requirement
}

pub fn calculate_heat(side_heat: &[f32]) -> f32 {
    side_heat.iter().copied().sum()
}

pub fn calculate_heat_excluding(side_heat: &[f32], came_from: &[usize]) -> f32 {
    side_heat
        .iter()
        .enumerate()
        .filter(|(index, _)| !came_from.contains(index))
        .map(|(_, value)| *value)
        .sum()
}

pub fn valid_heat_neighbor(
    rotatable: bool,
    split: bool,
    relative_to_neighbor: i32,
    neighbor_rotation: i32,
) -> bool {
    !rotatable
        || (!split && relative_to_neighbor == neighbor_rotation)
        || (split && relative_to_neighbor == (neighbor_rotation + 2).rem_euclid(4))
}

pub fn heat_contact_points(self_size: i32, other_size: i32, diff_tiles: f32) -> i32 {
    ((self_size as f32 / 2.0 + other_size as f32 / 2.0 - diff_tiles) as i32)
        .min(self_size.min(other_size))
        .max(0)
}

pub fn heat_contribution(
    neighbor_heat: f32,
    neighbor_size: i32,
    contact_points: i32,
    split: bool,
) -> f32 {
    let base = neighbor_heat / neighbor_size as f32 * contact_points as f32;
    if split {
        base / 3.0
    } else {
        base
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatNeighbor {
    pub side: usize,
    pub heat: f32,
    pub size: i32,
    pub contact_points: i32,
    pub split: bool,
    pub valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatAccumulation {
    pub heat: f32,
    pub side_heat: [f32; 4],
}

pub fn calculate_heat_from_neighbors(neighbors: &[HeatNeighbor]) -> HeatAccumulation {
    let mut result = HeatAccumulation {
        heat: 0.0,
        side_heat: [0.0; 4],
    };

    for neighbor in neighbors {
        if !neighbor.valid || neighbor.side >= 4 {
            continue;
        }
        let add = heat_contribution(
            neighbor.heat,
            neighbor.size,
            neighbor.contact_points,
            neighbor.split,
        );
        result.side_heat[neighbor.side] += add;
        result.heat += add;
    }

    result
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeatConductorState {
    pub heat: f32,
    pub side_heat: [f32; 4],
    pub last_heat_update: i64,
}

impl Default for HeatConductorState {
    fn default() -> Self {
        Self {
            heat: 0.0,
            side_heat: [0.0; 4],
            last_heat_update: -1,
        }
    }
}

pub fn update_conductor_heat(
    state: &mut HeatConductorState,
    update_id: i64,
    came_from: &[usize],
) -> bool {
    if state.last_heat_update == update_id {
        return false;
    }

    state.last_heat_update = update_id;
    state.heat = calculate_heat_excluding(&state.side_heat, came_from);
    true
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatProducerState {
    pub heat: f32,
}

impl Default for HeatProducerState {
    fn default() -> Self {
        Self { heat: 0.0 }
    }
}

pub fn heat_producer_update(
    state: &mut HeatProducerState,
    heat_output: f32,
    efficiency: f32,
    warmup_rate: f32,
    delta: f32,
) -> f32 {
    state.heat = approach_delta(state.heat, heat_output * efficiency, warmup_rate * delta);
    state.heat
}

pub fn heat_producer_frac(heat: f32, heat_output: f32) -> f32 {
    heat / heat_output
}

pub fn write_heat_producer_state<W: Write>(
    write: &mut W,
    state: &HeatProducerState,
) -> io::Result<()> {
    write_f32(write, state.heat)
}

pub fn read_heat_producer_state<R: Read>(read: &mut R) -> io::Result<HeatProducerState> {
    Ok(HeatProducerState {
        heat: read_f32(read)?,
    })
}

fn approach_delta(from: f32, to: f32, amount: f32) -> f32 {
    if from < to {
        (from + amount).min(to)
    } else {
        (from - amount).max(to)
    }
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_block_fraction_and_consumer_requirement_follow_java_contracts() {
        assert_eq!(heat_frac(15.0, 15.0, false), 1.0);
        assert_eq!(heat_frac(15.0, 15.0, true), 1.0 / 3.0);
        assert!(heat_consumer_requirement_met(10.0, 10.0));
        assert!(!heat_consumer_requirement_met(9.99, 10.0));
        assert!(heat_consumer_requirement_met(0.0, 0.0));

        let state = HeatBlockState {
            heat: 7.5,
            visual_max_heat: 15.0,
            split_heat: false,
        };
        assert_eq!(state.heat_frac(), 0.5);
    }

    #[test]
    fn heat_conductor_updates_once_per_update_id_and_excludes_sources() {
        assert_eq!(calculate_heat(&[1.0, 2.0, 3.0, 4.0]), 10.0);
        assert_eq!(
            calculate_heat_excluding(&[1.0, 2.0, 3.0, 4.0], &[1, 3]),
            4.0
        );

        let mut state = HeatConductorState {
            side_heat: [1.0, 2.0, 3.0, 4.0],
            ..Default::default()
        };
        assert!(update_conductor_heat(&mut state, 42, &[1]));
        assert_eq!(state.heat, 8.0);
        state.side_heat = [100.0; 4];
        assert!(!update_conductor_heat(&mut state, 42, &[]));
        assert_eq!(state.heat, 8.0);
        assert!(update_conductor_heat(&mut state, 43, &[]));
        assert_eq!(state.heat, 400.0);
    }

    #[test]
    fn heat_neighbor_orientation_contact_and_contribution_match_building_formula() {
        assert!(valid_heat_neighbor(false, false, 0, 2));
        assert!(valid_heat_neighbor(true, false, 1, 1));
        assert!(!valid_heat_neighbor(true, false, 1, 3));
        assert!(valid_heat_neighbor(true, true, 3, 1));
        assert!(!valid_heat_neighbor(true, true, 1, 1));

        assert_eq!(heat_contact_points(3, 3, 0.0), 3);
        assert_eq!(heat_contact_points(3, 3, 1.0), 2);
        assert_eq!(heat_contact_points(1, 3, 3.0), 0);
        assert_eq!(heat_contribution(12.0, 3, 2, false), 8.0);
        assert_eq!(heat_contribution(12.0, 3, 2, true), 8.0 / 3.0);

        let accumulated = calculate_heat_from_neighbors(&[
            HeatNeighbor {
                side: 0,
                heat: 12.0,
                size: 3,
                contact_points: 2,
                split: false,
                valid: true,
            },
            HeatNeighbor {
                side: 0,
                heat: 9.0,
                size: 3,
                contact_points: 3,
                split: true,
                valid: true,
            },
            HeatNeighbor {
                side: 2,
                heat: 99.0,
                size: 3,
                contact_points: 3,
                split: false,
                valid: false,
            },
        ]);
        assert_eq!(accumulated.side_heat[0], 11.0);
        assert_eq!(accumulated.side_heat[2], 0.0);
        assert_eq!(accumulated.heat, 11.0);
    }

    #[test]
    fn heat_producer_approaches_target_and_serializes_heat_after_super_state() {
        let mut state = HeatProducerState::default();
        assert_eq!(heat_producer_update(&mut state, 10.0, 1.0, 0.15, 2.0), 0.3);
        assert_eq!(heat_producer_update(&mut state, 10.0, 0.0, 0.15, 1.0), 0.15);
        assert_eq!(heat_producer_frac(5.0, 10.0), 0.5);

        let mut bytes = Vec::new();
        write_heat_producer_state(&mut bytes, &HeatProducerState { heat: 3.25 }).unwrap();
        assert_eq!(
            read_heat_producer_state(&mut bytes.as_slice()).unwrap(),
            HeatProducerState { heat: 3.25 }
        );
    }
}
