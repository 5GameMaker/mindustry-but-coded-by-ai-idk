//! Sync component mirroring upstream `mindustry.entities.comp.SyncComp`.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncHooks {
    pub snap_sync_calls: usize,
    pub snap_interpolation_calls: usize,
    pub read_sync_calls: usize,
    pub write_sync_calls: usize,
    pub read_sync_manual_calls: usize,
    pub write_sync_manual_calls: usize,
    pub after_sync_calls: usize,
    pub interpolate_calls: usize,
    pub handle_sync_hidden_calls: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncComp {
    pub last_updated: i64,
    pub update_spacing: i64,
    pub hooks: SyncHooks,
}

impl SyncComp {
    pub fn snap_sync(&mut self) {
        self.hooks.snap_sync_calls += 1;
    }

    pub fn snap_interpolation(&mut self) {
        self.hooks.snap_interpolation_calls += 1;
    }

    pub fn read_sync(&mut self) {
        self.hooks.read_sync_calls += 1;
    }

    pub fn write_sync(&mut self) {
        self.hooks.write_sync_calls += 1;
    }

    pub fn read_sync_manual(&mut self) {
        self.hooks.read_sync_manual_calls += 1;
    }

    pub fn write_sync_manual(&mut self) {
        self.hooks.write_sync_manual_calls += 1;
    }

    pub fn after_sync(&mut self) {
        self.hooks.after_sync_calls += 1;
    }

    pub fn interpolate(&mut self) {
        self.hooks.interpolate_calls += 1;
    }

    pub fn is_sync_hidden(&self, _player_id: i32) -> bool {
        false
    }

    pub fn handle_sync_hidden(&mut self) {
        self.hooks.handle_sync_hidden_calls += 1;
    }

    /// Java update condition:
    /// `(Vars.net.client() && !isLocal()) || isRemote()`.
    pub fn update(&mut self, net_client: bool, is_local: bool, is_remote: bool) {
        if (net_client && !is_local) || is_remote {
            self.interpolate();
        }
    }

    /// Java remove branch notifies the client of removed entity IDs.
    pub fn remove(&self, net_client: bool, entity_id: i32) -> Option<i32> {
        net_client.then_some(entity_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_component_update_interpolates_for_nonlocal_client_or_remote_player() {
        let mut sync = SyncComp::default();

        sync.update(true, false, false);
        sync.update(false, false, true);
        sync.update(true, true, false);

        assert_eq!(sync.hooks.interpolate_calls, 2);
    }

    #[test]
    fn sync_component_remove_reports_removed_entity_only_on_client() {
        let sync = SyncComp::default();

        assert_eq!(sync.remove(true, 42), Some(42));
        assert_eq!(sync.remove(false, 42), None);
    }

    #[test]
    fn sync_component_generated_hook_bodies_are_observable_noops() {
        let mut sync = SyncComp::default();

        sync.snap_sync();
        sync.snap_interpolation();
        sync.read_sync();
        sync.write_sync();
        sync.read_sync_manual();
        sync.write_sync_manual();
        sync.after_sync();
        sync.handle_sync_hidden();

        assert!(!sync.is_sync_hidden(7));
        assert_eq!(sync.hooks.snap_sync_calls, 1);
        assert_eq!(sync.hooks.snap_interpolation_calls, 1);
        assert_eq!(sync.hooks.read_sync_calls, 1);
        assert_eq!(sync.hooks.write_sync_calls, 1);
        assert_eq!(sync.hooks.read_sync_manual_calls, 1);
        assert_eq!(sync.hooks.write_sync_manual_calls, 1);
        assert_eq!(sync.hooks.after_sync_calls, 1);
        assert_eq!(sync.hooks.handle_sync_hidden_calls, 1);
    }
}
