use std::io::{self, Read, Write};

use crate::mindustry::io::{
    read_vec2,
    type_io::{read_i16, write_i16},
    write_vec2, Vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageBlockConfig {
    pub core_merge: bool,
    pub item_capacity: i32,
}

impl Default for StorageBlockConfig {
    fn default() -> Self {
        Self {
            core_merge: true,
            item_capacity: 0,
        }
    }
}

pub fn storage_accept_item(
    linked_core: bool,
    linked_core_accepts: bool,
    stored: i32,
    maximum_accepted: i32,
) -> bool {
    if linked_core {
        linked_core_accepts
    } else {
        stored < maximum_accepted
    }
}

pub fn storage_maximum_accepted(
    linked_core: bool,
    linked_core_max: i32,
    item_capacity: i32,
) -> i32 {
    if linked_core {
        linked_core_max
    } else {
        item_capacity
    }
}

pub fn storage_explosion_item_cap(linked_core: bool, item_capacity: i32) -> i32 {
    if linked_core {
        (item_capacity / 60).min(6)
    } else {
        item_capacity
    }
}

pub fn storage_can_pickup(linked_core: bool) -> bool {
    !linked_core
}

pub fn storage_allow_deposit(linked_core: bool, base_allow_deposit: bool) -> bool {
    linked_core || base_allow_deposit
}

pub fn storage_overwrite_clamped_items(
    own: &[i32],
    previous_modules: &[Vec<i32>],
    item_capacity: i32,
) -> Vec<i32> {
    let mut result = own.to_vec();
    for module in previous_modules {
        if result.len() < module.len() {
            result.resize(module.len(), 0);
        }
        for (index, amount) in module.iter().enumerate() {
            result[index] += *amount;
        }
    }
    for amount in &mut result {
        *amount = (*amount).min(item_capacity);
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreBlockConfig {
    pub is_first_tier: bool,
    pub allow_spawn: bool,
    pub requires_core_zone: bool,
    pub incinerate_non_buildable: bool,
    pub item_capacity: i32,
    pub size: i32,
}

impl Default for CoreBlockConfig {
    fn default() -> Self {
        Self {
            is_first_tier: false,
            allow_spawn: true,
            requires_core_zone: false,
            incinerate_non_buildable: false,
            item_capacity: 0,
            size: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoreBuildState {
    pub storage_capacity: i32,
    pub no_effect: bool,
    pub iframes: f32,
    pub thruster_time: f32,
    pub command_pos: Option<Vec2>,
}

impl Default for CoreBuildState {
    fn default() -> Self {
        Self {
            storage_capacity: 0,
            no_effect: false,
            iframes: -1.0,
            thruster_time: 0.0,
            command_pos: None,
        }
    }
}

pub fn core_light_radius(size: i32) -> f32 {
    30.0 + 20.0 * size as f32
}

pub fn core_fog_radius(current_fog_radius: i32, light_radius: f32) -> i32 {
    current_fog_radius.max((light_radius / 8.0 * 3.0) as i32 + 13)
}

#[allow(clippy::too_many_arguments)]
pub fn core_can_place_on(
    tile_exists: bool,
    editor: bool,
    linked_tiles_allow_core_placement: bool,
    linked_tiles_contain_core: bool,
    has_core: bool,
    infinite_resources: bool,
    has_requirements: bool,
    replacing_core: bool,
    new_size: i32,
    old_size: i32,
    requires_core_zone: bool,
) -> bool {
    if !tile_exists {
        return false;
    }
    if editor {
        return true;
    }
    if linked_tiles_allow_core_placement && !linked_tiles_contain_core {
        return true;
    }
    if !has_core || (!infinite_resources && !has_requirements) {
        return false;
    }
    replacing_core
        && new_size > old_size
        && (!requires_core_zone || linked_tiles_allow_core_placement)
}

pub fn core_can_replace(
    base_can_replace: bool,
    other_is_core: bool,
    new_size: i32,
    old_size: i32,
) -> bool {
    base_can_replace || (other_is_core && new_size >= old_size)
}

pub fn core_can_unload(block_unloadable: bool, allow_core_unloaders: bool) -> bool {
    block_unloadable && allow_core_unloaders
}

pub fn core_accept_item(core_incinerates: bool, stored: i32, maximum_accepted: i32) -> bool {
    core_incinerates || stored < maximum_accepted
}

pub fn core_maximum_accepted(core_incinerates: bool, storage_capacity: i32) -> i32 {
    if core_incinerates {
        i32::MAX / 2
    } else {
        storage_capacity
    }
}

pub fn core_handle_stack_amount(
    amount: i32,
    stored: i32,
    storage_capacity: i32,
    incinerate_non_buildable: bool,
    item_buildable: bool,
) -> i32 {
    let incinerate = incinerate_non_buildable && !item_buildable;
    if incinerate {
        0
    } else {
        amount.min(storage_capacity - stored).max(0)
    }
}

pub fn core_on_removed_storage_split(
    core_items: &[i32],
    storage_capacity: i32,
    total_capacity: i32,
) -> Vec<i32> {
    if total_capacity <= 0 {
        return vec![0; core_items.len()];
    }
    core_items
        .iter()
        .map(|amount| {
            (*amount as f32 * storage_capacity as f32 / total_capacity as f32)
                .min(storage_capacity as f32) as i32
        })
        .collect()
}

pub fn core_update_timers(iframes: &mut f32, thruster_time: &mut f32, delta: f32) {
    *iframes -= delta;
    *thruster_time -= delta / 90.0;
}

pub fn write_core_state<W: Write>(write: &mut W, state: &CoreBuildState) -> io::Result<()> {
    write_vec_nullable(write, state.command_pos)
}

pub fn read_core_state<R: Read>(read: &mut R, revision: i32) -> io::Result<CoreBuildState> {
    Ok(CoreBuildState {
        command_pos: if revision >= 1 {
            read_vec_nullable(read)?
        } else {
            None
        },
        ..CoreBuildState::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContainerStat {
    pub can_load: bool,
    pub can_unload: bool,
    pub not_storage: bool,
    pub load_factor: f32,
    pub last_used: i32,
}

pub fn unloader_possible_item(stats: &mut [ContainerStat]) -> bool {
    let mut has_provider = false;
    let mut has_receiver = false;
    let mut is_distinct = false;
    for stat in stats {
        is_distinct |= (has_provider && stat.can_load) || (has_receiver && stat.can_unload);
        has_provider |= stat.can_unload;
        has_receiver |= stat.can_load;
    }
    is_distinct
}

pub fn unloader_sort_key(stat: &ContainerStat) -> (bool, bool, bool, OrderedF32, i32) {
    (
        !stat.not_storage,
        stat.can_unload && !stat.can_load,
        stat.can_unload || !stat.can_load,
        OrderedF32(stat.load_factor),
        -stat.last_used,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnloaderUpdate {
    pub should_trade: bool,
    pub unload_timer: f32,
    pub rotations: i32,
}

pub fn unloader_update_timer(
    unload_timer: f32,
    delta: f32,
    speed: f32,
    possible_blocks: usize,
    traded: bool,
) -> UnloaderUpdate {
    let mut timer = unload_timer + delta;
    if timer < speed || possible_blocks < 2 {
        return UnloaderUpdate {
            should_trade: false,
            unload_timer: timer,
            rotations: 0,
        };
    }
    if traded {
        timer %= speed;
    } else {
        timer = timer.min(speed);
    }
    UnloaderUpdate {
        should_trade: traded,
        unload_timer: timer,
        rotations: 0,
    }
}

pub fn write_unloader_sort_item<W: Write>(write: &mut W, sort_item: Option<i32>) -> io::Result<()> {
    write_i16(write, sort_item.unwrap_or(-1) as i16)
}

pub fn read_unloader_sort_item<R: Read>(read: &mut R, revision: i32) -> io::Result<Option<i32>> {
    let id = if revision == 1 {
        read_i16(read)? as i32
    } else {
        read_i8(read)? as i32
    };
    Ok((id != -1).then_some(id))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrderedF32(pub f32);

impl Eq for OrderedF32 {}

impl PartialOrd for OrderedF32 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
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

fn read_i8<R: Read>(read: &mut R) -> io::Result<i8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0] as i8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_block_helpers_follow_linked_core_rules() {
        assert!(storage_accept_item(false, false, 4, 5));
        assert!(!storage_accept_item(false, true, 5, 5));
        assert!(storage_accept_item(true, true, 99, 0));
        assert_eq!(storage_maximum_accepted(true, 1000, 50), 1000);
        assert_eq!(storage_explosion_item_cap(true, 600), 6);
        assert_eq!(storage_explosion_item_cap(true, 120), 2);
        assert_eq!(storage_explosion_item_cap(false, 120), 120);
        assert!(!storage_can_pickup(true));
        assert!(storage_allow_deposit(true, false));

        assert_eq!(
            storage_overwrite_clamped_items(&[1, 9], &[vec![5, 5], vec![100, 0]], 10),
            vec![10, 10]
        );
    }

    #[test]
    fn core_block_placement_capacity_and_timer_helpers_match_upstream() {
        assert_eq!(core_light_radius(3), 90.0);
        assert_eq!(core_fog_radius(0, 90.0), 46);
        assert!(!core_can_place_on(
            false, false, true, false, true, true, true, false, 3, 2, false
        ));
        assert!(core_can_place_on(
            true, true, false, true, false, false, false, false, 3, 2, false
        ));
        assert!(core_can_place_on(
            true, false, true, false, false, false, false, false, 3, 2, false
        ));
        assert!(core_can_place_on(
            true, false, true, true, true, false, true, true, 3, 2, true
        ));
        assert!(!core_can_place_on(
            true, false, false, true, true, false, true, true, 3, 2, true
        ));

        assert!(core_can_replace(false, true, 3, 3));
        assert!(core_can_unload(true, true));
        assert!(core_accept_item(true, 999, 1));
        assert_eq!(core_maximum_accepted(true, 100), i32::MAX / 2);
        assert_eq!(core_handle_stack_amount(10, 95, 100, false, true), 5);
        assert_eq!(core_handle_stack_amount(10, 0, 100, true, false), 0);
        assert_eq!(
            core_on_removed_storage_split(&[100, 50], 20, 100),
            vec![20, 10]
        );

        let mut iframes = 10.0;
        let mut thruster = 1.0;
        core_update_timers(&mut iframes, &mut thruster, 5.0);
        assert_eq!(iframes, 5.0);
        assert!((thruster - 0.9444444).abs() < 0.00001);
    }

    #[test]
    fn core_and_unloader_state_serialization_follows_java_revision_layout() {
        let state = CoreBuildState {
            command_pos: Some(Vec2::new(1.5, -2.25)),
            ..CoreBuildState::default()
        };
        let mut bytes = Vec::new();
        write_core_state(&mut bytes, &state).unwrap();
        assert_eq!(read_core_state(&mut bytes.as_slice(), 1).unwrap(), state);
        assert_eq!(
            read_core_state(&mut bytes.as_slice(), 0).unwrap(),
            CoreBuildState::default()
        );

        let mut bytes = Vec::new();
        write_core_state(&mut bytes, &CoreBuildState::default()).unwrap();
        assert_eq!(
            read_core_state(&mut bytes.as_slice(), 1)
                .unwrap()
                .command_pos,
            None
        );

        let mut bytes = Vec::new();
        write_unloader_sort_item(&mut bytes, Some(300)).unwrap();
        assert_eq!(bytes, 300i16.to_be_bytes());
        assert_eq!(
            read_unloader_sort_item(&mut bytes.as_slice(), 1).unwrap(),
            Some(300)
        );
        assert_eq!(
            read_unloader_sort_item(&mut [255u8].as_slice(), 0).unwrap(),
            None
        );
    }

    #[test]
    fn unloader_item_selection_and_sorting_match_java_priority_order() {
        let mut stats = [
            ContainerStat {
                can_load: false,
                can_unload: true,
                not_storage: true,
                load_factor: 0.8,
                last_used: 1,
            },
            ContainerStat {
                can_load: true,
                can_unload: false,
                not_storage: true,
                load_factor: 0.2,
                last_used: 2,
            },
        ];
        assert!(unloader_possible_item(&mut stats));

        let mut sorted = vec![
            ContainerStat {
                can_load: true,
                can_unload: true,
                not_storage: true,
                load_factor: 0.5,
                last_used: 1,
            },
            ContainerStat {
                can_load: true,
                can_unload: false,
                not_storage: true,
                load_factor: 0.1,
                last_used: 2,
            },
            ContainerStat {
                can_load: false,
                can_unload: true,
                not_storage: false,
                load_factor: 0.9,
                last_used: 3,
            },
        ];
        sorted.sort_by_key(unloader_sort_key);
        assert!(sorted[0].can_load && !sorted[0].can_unload);
        assert!(!sorted[2].not_storage);

        assert_eq!(
            unloader_update_timer(0.5, 0.4, 1.0, 2, false).unload_timer,
            0.9
        );
        assert!((unloader_update_timer(0.9, 0.4, 1.0, 2, true).unload_timer - 0.3).abs() < 0.00001);
        assert_eq!(
            unloader_update_timer(0.9, 0.4, 1.0, 2, false).unload_timer,
            1.0
        );
    }
}
