use std::io::{self, Read, Write};

use crate::mindustry::io::{
    read_vec2,
    type_io::{read_i16, write_i16},
    write_vec2, Vec2,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitBlockState {
    pub progress: f32,
    pub time: f32,
    pub speed_scl: f32,
    pub has_payload: bool,
}

impl Default for UnitBlockState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            time: 0.0,
            speed_scl: 0.0,
            has_payload: false,
        }
    }
}

pub fn unit_block_spawned(state: &mut UnitBlockState) {
    state.progress = 0.0;
    state.has_payload = false;
}

pub fn unit_block_dump_payload(dumped: bool, state: &mut UnitBlockState) -> bool {
    if dumped {
        unit_block_spawned(state);
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitPlanRequirement {
    pub item_id: usize,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPlan {
    pub unit_id: i32,
    pub time: f32,
    pub requirements: Vec<UnitPlanRequirement>,
    pub banned: bool,
}

pub fn unit_factory_capacities(item_count: usize, plans: &[UnitPlan]) -> (Vec<i32>, i32) {
    let mut capacities = vec![0; item_count];
    let mut item_capacity = 10;
    for plan in plans {
        for stack in &plan.requirements {
            if stack.item_id < capacities.len() {
                capacities[stack.item_id] = capacities[stack.item_id].max(stack.amount * 2);
            }
            item_capacity = item_capacity.max(stack.amount * 2);
        }
    }
    (capacities, item_capacity)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitFactoryState {
    pub base: UnitBlockState,
    pub current_plan: i32,
    pub command_pos: Option<Vec2>,
    pub command_id: Option<u8>,
}

impl Default for UnitFactoryState {
    fn default() -> Self {
        Self {
            base: UnitBlockState::default(),
            current_plan: -1,
            command_pos: None,
            command_id: None,
        }
    }
}

pub fn unit_factory_configure_plan(
    state: &mut UnitFactoryState,
    requested: i32,
    plan_count: usize,
    command_valid_for_next_unit: bool,
) {
    if state.current_plan == requested {
        return;
    }
    state.current_plan = if requested < 0 || requested as usize >= plan_count {
        -1
    } else {
        requested
    };
    state.base.progress = 0.0;
    if !command_valid_for_next_unit {
        state.command_id = None;
    }
}

pub fn unit_factory_fraction(current_plan: i32, progress: f32, plan_time: f32) -> f32 {
    if current_plan == -1 || plan_time == 0.0 {
        0.0
    } else {
        progress / plan_time
    }
}

pub fn unit_factory_should_consume(
    current_plan: i32,
    enabled: bool,
    has_payload: bool,
    team_activates_factories: bool,
) -> bool {
    current_plan != -1 && enabled && !has_payload && team_activates_factories
}

pub fn unit_factory_accept_item(
    current_plan: i32,
    stored: i32,
    maximum_accepted: i32,
    plan_contains_item: bool,
) -> bool {
    current_plan != -1 && stored < maximum_accepted && plan_contains_item
}

pub fn unit_factory_maximum_accepted(base_capacity: i32, unit_cost: f32) -> i32 {
    (base_capacity as f32 * unit_cost).round() as i32
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitFactoryUpdate {
    pub created_unit: bool,
    pub progress: f32,
    pub time: f32,
    pub speed_scl: f32,
    pub current_plan: i32,
}

#[allow(clippy::too_many_arguments)]
pub fn unit_factory_update(
    state: &mut UnitFactoryState,
    configurable: bool,
    plan_count: usize,
    efficiency: f32,
    edelta: f32,
    unit_build_speed: f32,
    plan_time: f32,
    plan_banned: bool,
) -> UnitFactoryUpdate {
    if !configurable {
        state.current_plan = 0;
    }
    if state.current_plan < 0 || state.current_plan as usize >= plan_count {
        state.current_plan = -1;
    }

    if efficiency > 0.0 && state.current_plan != -1 {
        state.base.time += edelta * state.base.speed_scl * unit_build_speed;
        state.base.progress += edelta * unit_build_speed;
        state.base.speed_scl = lerp_delta(state.base.speed_scl, 1.0, 0.05);
    } else {
        state.base.speed_scl = lerp_delta(state.base.speed_scl, 0.0, 0.05);
    }

    let mut created_unit = false;
    if state.current_plan != -1 && !state.base.has_payload {
        if plan_banned {
            state.current_plan = -1;
        } else if state.base.progress >= plan_time {
            state.base.progress %= 1.0;
            state.base.has_payload = true;
            created_unit = true;
        }
        state.base.progress = state.base.progress.clamp(0.0, plan_time);
    } else {
        state.base.progress = 0.0;
    }

    UnitFactoryUpdate {
        created_unit,
        progress: state.base.progress,
        time: state.base.time,
        speed_scl: state.base.speed_scl,
        current_plan: state.current_plan,
    }
}

pub fn write_unit_factory_state<W: Write>(
    write: &mut W,
    state: &UnitFactoryState,
) -> io::Result<()> {
    write_f32(write, state.base.progress)?;
    write_i16(write, state.current_plan as i16)?;
    write_vec_nullable(write, state.command_pos)?;
    write_command(write, state.command_id)
}

pub fn read_unit_factory_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<UnitFactoryState> {
    let progress = read_f32(read)?;
    let current_plan = read_i16(read)? as i32;
    let command_pos = if revision >= 2 {
        read_vec_nullable(read)?
    } else {
        None
    };
    let command_id = if revision >= 3 {
        read_command(read)?
    } else {
        None
    };
    Ok(UnitFactoryState {
        base: UnitBlockState {
            progress,
            ..UnitBlockState::default()
        },
        current_plan,
        command_pos,
        command_id,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairTowerState {
    pub refresh: f32,
    pub warmup: f32,
    pub total_progress: f32,
}

impl Default for RepairTowerState {
    fn default() -> Self {
        Self {
            refresh: 0.0,
            warmup: 0.0,
            total_progress: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairTowerUpdate {
    pub should_refresh_targets: bool,
    pub should_consume: bool,
    pub heal_per_target: f32,
}

pub fn repair_tower_update(
    state: &mut RepairTowerState,
    potential_efficiency: f32,
    efficiency: f32,
    suppressed: bool,
    target_count: usize,
    damaged_target_count: usize,
    heal_amount: f32,
    delta: f32,
    circle_speed: f32,
) -> RepairTowerUpdate {
    let mut should_refresh_targets = false;
    if potential_efficiency > 0.0 {
        state.refresh += delta;
        if state.refresh >= REPAIR_TOWER_REFRESH_INTERVAL {
            state.refresh = 0.0;
            should_refresh_targets = true;
        }
    }

    if suppressed {
        state.warmup = 0.0;
        return RepairTowerUpdate {
            should_refresh_targets,
            should_consume: target_count > 0,
            heal_per_target: 0.0,
        };
    }

    let any = efficiency > 0.0 && damaged_target_count > 0;
    let heal_per_target = if efficiency > 0.0 {
        heal_amount * delta * efficiency
    } else {
        0.0
    };
    state.warmup = lerp_delta(state.warmup, if any { efficiency } else { 0.0 }, 0.08);
    state.total_progress += delta / circle_speed;

    RepairTowerUpdate {
        should_refresh_targets,
        should_consume: target_count > 0,
        heal_per_target,
    }
}

pub const REPAIR_TOWER_REFRESH_INTERVAL: f32 = 6.0;

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn write_command<W: Write>(write: &mut W, command_id: Option<u8>) -> io::Result<()> {
    write.write_all(&[command_id.unwrap_or(255)])
}

fn read_command<R: Read>(read: &mut R) -> io::Result<Option<u8>> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok((buf[0] != 255).then_some(buf[0]))
}

fn write_vec_nullable<W: Write>(write: &mut W, value: Option<Vec2>) -> io::Result<()> {
    match value {
        Some(value) => write_vec2(write, value),
        None => {
            write.write_all(&f32::NAN.to_be_bytes())?;
            write.write_all(&f32::NAN.to_be_bytes())
        }
    }
}

fn read_vec_nullable<R: Read>(read: &mut R) -> io::Result<Option<Vec2>> {
    let vec = read_vec2(read)?;
    Ok((!vec.x.is_nan() && !vec.y.is_nan()).then_some(vec))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_block_spawn_and_dump_reset_payload_state_like_upstream() {
        let mut state = UnitBlockState {
            progress: 10.0,
            time: 3.0,
            speed_scl: 0.5,
            has_payload: true,
        };
        unit_block_spawned(&mut state);
        assert_eq!(state.progress, 0.0);
        assert!(!state.has_payload);

        state.has_payload = true;
        state.progress = 5.0;
        assert!(unit_block_dump_payload(true, &mut state));
        assert_eq!(state.progress, 0.0);
        assert!(!state.has_payload);
    }

    #[test]
    fn unit_factory_capacities_config_and_acceptance_follow_upstream() {
        let plans = vec![
            UnitPlan {
                unit_id: 1,
                time: 60.0,
                requirements: vec![
                    UnitPlanRequirement {
                        item_id: 0,
                        amount: 20,
                    },
                    UnitPlanRequirement {
                        item_id: 1,
                        amount: 5,
                    },
                ],
                banned: false,
            },
            UnitPlan {
                unit_id: 2,
                time: 90.0,
                requirements: vec![UnitPlanRequirement {
                    item_id: 0,
                    amount: 30,
                }],
                banned: false,
            },
        ];
        let (capacities, item_capacity) = unit_factory_capacities(3, &plans);
        assert_eq!(capacities, vec![60, 10, 0]);
        assert_eq!(item_capacity, 60);

        let mut state = UnitFactoryState {
            current_plan: 1,
            command_id: Some(7),
            ..UnitFactoryState::default()
        };
        unit_factory_configure_plan(&mut state, 9, plans.len(), false);
        assert_eq!(state.current_plan, -1);
        assert_eq!(state.base.progress, 0.0);
        assert_eq!(state.command_id, None);

        assert_eq!(unit_factory_fraction(-1, 10.0, 60.0), 0.0);
        assert_eq!(unit_factory_fraction(0, 30.0, 60.0), 0.5);
        assert!(unit_factory_should_consume(0, true, false, true));
        assert!(!unit_factory_should_consume(0, true, true, true));
        assert!(unit_factory_accept_item(0, 4, 5, true));
        assert!(!unit_factory_accept_item(0, 4, 5, false));
        assert_eq!(unit_factory_maximum_accepted(10, 1.5), 15);
    }

    #[test]
    fn unit_factory_update_and_serialization_match_revision_three_order() {
        let mut state = UnitFactoryState {
            current_plan: 0,
            command_pos: Some(Vec2::new(3.0, 4.0)),
            command_id: Some(2),
            ..UnitFactoryState::default()
        };
        state.base.progress = 58.5;
        state.base.speed_scl = 0.0;
        let update = unit_factory_update(&mut state, true, 1, 1.0, 1.0, 1.0, 60.0, false);
        assert!(!update.created_unit);
        assert_eq!(update.progress, 59.5);
        assert_eq!(update.speed_scl, 0.05);

        let update = unit_factory_update(&mut state, true, 1, 1.0, 1.0, 1.0, 60.0, false);
        assert!(update.created_unit);
        assert!(state.base.has_payload);
        assert!(state.base.progress < 1.0);

        let mut bytes = Vec::new();
        write_unit_factory_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_unit_factory_state(&mut bytes.as_slice(), 3).unwrap(),
            UnitFactoryState {
                base: UnitBlockState {
                    progress: state.base.progress,
                    ..UnitBlockState::default()
                },
                current_plan: state.current_plan,
                command_pos: state.command_pos,
                command_id: state.command_id,
            }
        );

        let legacy = [
            1.0f32.to_be_bytes().as_slice(),
            3i16.to_be_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            read_unit_factory_state(&mut legacy.as_slice(), 1).unwrap(),
            UnitFactoryState {
                base: UnitBlockState {
                    progress: 1.0,
                    ..UnitBlockState::default()
                },
                current_plan: 3,
                command_pos: None,
                command_id: None,
            }
        );
    }

    #[test]
    fn repair_tower_refresh_heal_warmup_and_consumption_follow_upstream() {
        let mut state = RepairTowerState {
            refresh: 5.5,
            warmup: 0.0,
            total_progress: 0.0,
        };
        let update = repair_tower_update(&mut state, 1.0, 0.75, false, 2, 1, 3.0, 1.0, 120.0);
        assert!(update.should_refresh_targets);
        assert!(update.should_consume);
        assert_eq!(update.heal_per_target, 2.25);
        assert_eq!(state.refresh, 0.0);
        assert_eq!(state.warmup, 0.06);
        assert!((state.total_progress - 1.0 / 120.0).abs() < 0.00001);

        let update = repair_tower_update(&mut state, 1.0, 1.0, true, 1, 1, 3.0, 1.0, 120.0);
        assert_eq!(state.warmup, 0.0);
        assert_eq!(update.heal_per_target, 0.0);
    }
}
