use std::io::{self, Read, Write};

use crate::mindustry::world::{
    meta::{BlockFlag, BlockGroup, Env},
    Block,
};

use super::{
    core_accept_item, core_can_place_on, core_can_replace, core_can_unload, core_fog_radius,
    core_handle_stack_amount, core_light_radius, core_maximum_accepted,
    core_on_removed_storage_split, core_update_timers, read_core_state, read_unloader_sort_item,
    storage_accept_item, storage_allow_deposit, storage_can_pickup, storage_explosion_item_cap,
    storage_maximum_accepted, storage_overwrite_clamped_items, unloader_possible_item,
    unloader_sort_key, unloader_update_timer, write_core_state, write_unloader_sort_item,
    ContainerStat, CoreBlockConfig, CoreBuildState, OrderedF32, UnloaderUpdate,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StorageBlock {
    pub block: Block,
    pub core_merge: bool,
    pub separate_item_capacity: bool,
    pub allow_resupply: bool,
}

impl StorageBlock {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.has_items = true;
        block.solid = true;
        block.update = false;
        block.sync = true;
        block.destructible = true;
        block.group = BlockGroup::Transportation;
        block.flags = vec![BlockFlag::Storage];
        block.env_enabled = Env::ANY;

        Self {
            block,
            core_merge: true,
            separate_item_capacity: true,
            allow_resupply: true,
        }
    }

    pub fn accept_item(
        &self,
        linked_core: bool,
        linked_core_accepts: bool,
        stored: i32,
        maximum_accepted: i32,
    ) -> bool {
        storage_accept_item(linked_core, linked_core_accepts, stored, maximum_accepted)
    }

    pub fn maximum_accepted(&self, linked_core: bool, linked_core_max: i32) -> i32 {
        storage_maximum_accepted(linked_core, linked_core_max, self.block.item_capacity)
    }

    pub fn explosion_item_cap(&self, linked_core: bool) -> i32 {
        storage_explosion_item_cap(linked_core, self.block.item_capacity)
    }

    pub fn can_pickup(&self, linked_core: bool) -> bool {
        storage_can_pickup(linked_core)
    }

    pub fn allow_deposit(&self, linked_core: bool, base_allow_deposit: bool) -> bool {
        storage_allow_deposit(linked_core, base_allow_deposit)
    }

    pub fn overwrite_clamped_items(&self, own: &[i32], previous_modules: &[Vec<i32>]) -> Vec<i32> {
        storage_overwrite_clamped_items(own, previous_modules, self.block.item_capacity)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreBlock {
    pub storage: StorageBlock,
    pub config: CoreBlockConfig,
    pub unit_cap_modifier: i32,
    pub always_allow_deposit: bool,
}

impl CoreBlock {
    pub fn new(name: impl Into<String>) -> Self {
        let mut storage = StorageBlock::new(name);
        storage.block.update = true;
        storage.block.sync = false;
        storage.block.commandable = true;
        storage.block.replaceable = false;
        storage.block.flags = vec![BlockFlag::Core];
        storage.block.group = BlockGroup::None;
        storage.block.env_enabled |= Env::SPACE;

        Self {
            storage,
            config: CoreBlockConfig::default(),
            unit_cap_modifier: 10,
            always_allow_deposit: true,
        }
    }

    pub fn light_radius(&self) -> f32 {
        core_light_radius(self.config.size)
    }

    pub fn fog_radius(&self, current_fog_radius: i32) -> i32 {
        core_fog_radius(current_fog_radius, self.light_radius())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn can_place_on(
        &self,
        tile_exists: bool,
        editor: bool,
        linked_tiles_allow_core_placement: bool,
        linked_tiles_contain_core: bool,
        has_core: bool,
        infinite_resources: bool,
        has_requirements: bool,
        replacing_core: bool,
        old_size: i32,
    ) -> bool {
        core_can_place_on(
            tile_exists,
            editor,
            linked_tiles_allow_core_placement,
            linked_tiles_contain_core,
            has_core,
            infinite_resources,
            has_requirements,
            replacing_core,
            self.config.size,
            old_size,
            self.config.requires_core_zone,
        )
    }

    pub fn can_replace(&self, base_can_replace: bool, other_is_core: bool, old_size: i32) -> bool {
        core_can_replace(base_can_replace, other_is_core, self.config.size, old_size)
    }

    pub fn can_unload(&self, block_unloadable: bool, allow_core_unloaders: bool) -> bool {
        core_can_unload(block_unloadable, allow_core_unloaders)
    }

    pub fn accept_item(&self, core_incinerates: bool, stored: i32, maximum_accepted: i32) -> bool {
        core_accept_item(core_incinerates, stored, maximum_accepted)
    }

    pub fn maximum_accepted(&self, core_incinerates: bool) -> i32 {
        core_maximum_accepted(core_incinerates, self.storage.block.item_capacity)
    }

    pub fn handle_stack_amount(
        &self,
        amount: i32,
        stored: i32,
        incinerate_non_buildable: bool,
        item_buildable: bool,
    ) -> i32 {
        core_handle_stack_amount(
            amount,
            stored,
            self.storage.block.item_capacity,
            incinerate_non_buildable,
            item_buildable,
        )
    }

    pub fn on_removed_storage_split(&self, core_items: &[i32], total_capacity: i32) -> Vec<i32> {
        core_on_removed_storage_split(core_items, self.storage.block.item_capacity, total_capacity)
    }

    pub fn update_timers(&self, iframes: &mut f32, thruster_time: &mut f32, delta: f32) {
        core_update_timers(iframes, thruster_time, delta)
    }

    pub fn write_state<W: Write>(&self, write: &mut W, state: &CoreBuildState) -> io::Result<()> {
        write_core_state(write, state)
    }

    pub fn read_state<R: Read>(&self, read: &mut R, revision: i32) -> io::Result<CoreBuildState> {
        read_core_state(read, revision)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unloader {
    pub block: Block,
    pub speed: f32,
    pub allow_core_unload: bool,
    pub no_update_disabled: bool,
}

impl Unloader {
    pub fn new(name: impl Into<String>) -> Self {
        let mut block = Block::new(0, name);
        block.update = true;
        block.solid = true;
        block.health = 70;
        block.has_items = true;
        block.configurable = true;
        block.save_config = true;
        block.item_capacity = 0;
        block.clear_on_double_tap = true;
        block.group = BlockGroup::Transportation;
        block.env_enabled = Env::ANY;
        block.destructible = true;

        Self {
            block,
            speed: 1.0,
            allow_core_unload: true,
            no_update_disabled: true,
        }
    }

    pub fn possible_item(&self, stats: &mut [ContainerStat]) -> bool {
        unloader_possible_item(stats)
    }

    pub fn sort_key(&self, stat: &ContainerStat) -> (bool, bool, bool, OrderedF32, i32) {
        unloader_sort_key(stat)
    }

    pub fn update_timer(
        &self,
        unload_timer: f32,
        delta: f32,
        possible_blocks: usize,
        traded: bool,
    ) -> UnloaderUpdate {
        unloader_update_timer(unload_timer, delta, self.speed, possible_blocks, traded)
    }

    pub fn write_sort_item<W: Write>(
        &self,
        write: &mut W,
        sort_item: Option<i32>,
    ) -> io::Result<()> {
        write_unloader_sort_item(write, sort_item)
    }

    pub fn read_sort_item<R: Read>(&self, read: &mut R, revision: i32) -> io::Result<Option<i32>> {
        read_unloader_sort_item(read, revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_block_constructor_matches_java_storage_defaults() {
        let storage = StorageBlock::new("container");

        assert_eq!(storage.block.name, "container");
        assert!(storage.block.has_items);
        assert!(storage.block.solid);
        assert!(!storage.block.update);
        assert!(storage.block.sync);
        assert!(storage.block.destructible);
        assert_eq!(storage.block.group, BlockGroup::Transportation);
        assert_eq!(storage.block.flags, vec![BlockFlag::Storage]);
        assert!(storage.core_merge);
        assert!(storage.separate_item_capacity);
        assert!(storage.allow_resupply);
    }

    #[test]
    fn core_block_constructor_and_helpers_follow_runtime_contract() {
        let core = CoreBlock::new("core");

        assert!(core.storage.block.update);
        assert!(!core.storage.block.sync);
        assert!(core.storage.block.commandable);
        assert_eq!(core.storage.block.flags, vec![BlockFlag::Core]);
        assert_eq!(core.config, CoreBlockConfig::default());
        assert_eq!(core.unit_cap_modifier, 10);
        assert!(core.always_allow_deposit);
        assert_eq!(core.light_radius(), 50.0);
        assert_eq!(core.fog_radius(0), 31);
        assert!(core.can_unload(true, true));
        assert!(core.accept_item(true, 999, 1));
        assert_eq!(core.maximum_accepted(true), i32::MAX / 2);
        assert_eq!(core.handle_stack_amount(10, 5, false, true), 5);
        assert_eq!(core.on_removed_storage_split(&[100, 50], 100), vec![10, 5]);
    }

    #[test]
    fn unloader_constructor_and_helpers_match_java_sorting_layout() {
        let unloader = Unloader::new("unloader");

        assert!(unloader.block.update);
        assert!(unloader.block.solid);
        assert_eq!(unloader.block.health, 70);
        assert!(unloader.block.has_items);
        assert!(unloader.block.configurable);
        assert!(unloader.block.save_config);
        assert_eq!(unloader.block.item_capacity, 0);
        assert!(unloader.no_update_disabled);
        assert!(unloader.allow_core_unload);

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
        assert!(unloader.possible_item(&mut stats));
        assert_eq!(
            unloader.sort_key(&stats[0]),
            (false, true, true, OrderedF32(0.8), -1)
        );
        assert_eq!(unloader.update_timer(0.9, 0.4, 2, false).unload_timer, 1.0);
    }
}
