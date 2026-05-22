//! Player component shell mirroring upstream `mindustry.entities.comp.PlayerComp`.
//!
//! This first-stage port keeps the player as a mostly pure data object with
//! explicit helpers for preview-plan buffering, identity formatting and a few
//! lightweight state checks. World-heavy behavior (`Vars`, `State`, `Events`,
//! drawing, network mutation, unit control) stays outside this file for now.

use crate::mindustry::ai::unit_command::UnitCommand;
use crate::mindustry::entities::units::BuildPlan;
use crate::mindustry::game::{CoreInfo, TEAM_SHARDED};
use crate::mindustry::io::{TeamId, UnitRef};
use crate::mindustry::net::NetConnection;
use crate::mindustry::world::block::Block;

const PREVIEW_PLAN_COMMIT_DELAY_MS: i64 = 100;
const LOCAL_PLAYER_IP: &str = "localhost";
const LOCAL_PLAYER_ID: &str = "[LOCAL]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerUnitState {
    pub reference: UnitRef,
    pub valid: bool,
    pub can_build: bool,
    pub block_unit: bool,
}

impl PlayerUnitState {
    pub const fn unit(id: i32) -> Self {
        Self {
            reference: UnitRef::Unit { id },
            valid: true,
            can_build: false,
            block_unit: false,
        }
    }

    pub const fn block(tile_pos: i32) -> Self {
        Self {
            reference: UnitRef::Block { tile_pos },
            valid: true,
            can_build: false,
            block_unit: true,
        }
    }

    pub fn with_valid(mut self, valid: bool) -> Self {
        self.valid = valid;
        self
    }

    pub fn with_can_build(mut self, can_build: bool) -> Self {
        self.can_build = can_build;
        self
    }

    pub fn with_block_unit(mut self, block_unit: bool) -> Self {
        self.block_unit = block_unit;
        self
    }

    pub fn is_dead(self) -> bool {
        !self.valid
    }
}

impl Default for PlayerUnitState {
    fn default() -> Self {
        Self {
            reference: UnitRef::Null,
            valid: false,
            can_build: false,
            block_unit: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerComp {
    pub x: f32,
    pub y: f32,
    pub unit: Option<PlayerUnitState>,
    pub con: Option<NetConnection>,
    pub team: TeamId,
    pub typing: bool,
    pub shooting: bool,
    pub boosting: bool,
    pub selected_block: Option<Block>,
    pub selected_rotation: i32,
    pub mouse_x: f32,
    pub mouse_y: f32,
    /// Command the unit had before it was controlled.
    pub last_command: Option<UnitCommand>,
    pub admin: bool,
    pub name: String,
    pub color: u32,
    pub locale: String,
    pub death_timer: f32,
    pub last_text: String,
    pub text_fade_time: f32,
    pub ping_x: f32,
    pub ping_y: f32,
    pub ping_time: f32,
    pub ping_text: Option<String>,
    pub last_read_unit: Option<PlayerUnitState>,
    pub wrong_read_units: i32,
    pub just_switch_from: Option<PlayerUnitState>,
    pub just_switch_to: Option<PlayerUnitState>,
    pub last_preview_plan_group: i32,
    pub last_preview_plan_group_server: i32,
    pub last_preview_plan_timestamp: i64,
    pub receiving_new_plan_group: bool,
    pub preview_plans_current: Vec<BuildPlan>,
    pub preview_plans_assembling: Vec<BuildPlan>,
    pub preview_plans_dirty: bool,
}

impl PlayerComp {
    pub fn new(team: TeamId) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            unit: None,
            con: None,
            team,
            typing: false,
            shooting: false,
            boosting: false,
            selected_block: None,
            selected_rotation: 0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            last_command: None,
            admin: false,
            name: "frog".into(),
            color: 0xffff_ffff,
            locale: "en".into(),
            death_timer: 0.0,
            last_text: String::new(),
            text_fade_time: 0.0,
            ping_x: 0.0,
            ping_y: 0.0,
            ping_time: 0.0,
            ping_text: None,
            last_read_unit: None,
            wrong_read_units: 0,
            just_switch_from: None,
            just_switch_to: None,
            last_preview_plan_group: -1,
            last_preview_plan_group_server: -1,
            last_preview_plan_timestamp: 0,
            receiving_new_plan_group: false,
            preview_plans_current: Vec::new(),
            preview_plans_assembling: Vec::new(),
            preview_plans_dirty: false,
        }
    }

    pub fn unit_ref(&self) -> Option<UnitRef> {
        self.unit.map(|unit| unit.reference)
    }

    pub fn set_unit_state(&mut self, unit: PlayerUnitState) {
        self.unit = Some(unit);
    }

    pub fn clear_unit(&mut self) {
        self.unit = None;
    }

    pub fn core_with<F>(&self, lookup: F) -> Option<CoreInfo>
    where
        F: FnOnce(TeamId) -> Option<CoreInfo>,
    {
        lookup(self.team)
    }

    pub fn closest_core_with<F>(&self, lookup: F) -> Option<CoreInfo>
    where
        F: FnOnce(f32, f32, TeamId) -> Option<CoreInfo>,
    {
        lookup(self.x, self.y, self.team)
    }

    pub fn get_preview_plans(&mut self, now_millis: i64) -> &[BuildPlan] {
        if self.receiving_new_plan_group
            && now_millis.saturating_sub(self.last_preview_plan_timestamp)
                >= PREVIEW_PLAN_COMMIT_DELAY_MS
        {
            self.receiving_new_plan_group = false;
            self.preview_plans_dirty = true;
            self.preview_plans_current.clear();
            self.preview_plans_current
                .extend_from_slice(&self.preview_plans_assembling);
            self.preview_plans_assembling.clear();
        }

        &self.preview_plans_current
    }

    pub fn handle_preview_plans(
        &mut self,
        group_id: i32,
        plans: &[BuildPlan],
        now_millis: i64,
        max_preview_plans: usize,
    ) {
        if group_id > self.last_preview_plan_group {
            self.preview_plans_assembling.clear();
            self.last_preview_plan_group = group_id;
            self.receiving_new_plan_group = true;
            self.last_preview_plan_timestamp = now_millis;
        } else if group_id < self.last_preview_plan_group || !self.receiving_new_plan_group {
            return;
        }

        let remaining = max_preview_plans.saturating_sub(self.preview_plans_assembling.len());
        let added = plans.len().min(remaining);
        if added > 0 {
            self.preview_plans_assembling
                .extend_from_slice(&plans[..added]);
        }
    }

    pub fn is_builder(&self) -> bool {
        self.unit
            .as_ref()
            .is_some_and(|unit| unit.valid && unit.can_build)
    }

    pub fn display_ammo(&self, unit_ammo_enabled: bool) -> bool {
        self.unit.as_ref().map_or(unit_ammo_enabled, |unit| {
            unit.block_unit || unit_ammo_enabled
        })
    }

    pub fn dead(&self) -> bool {
        self.unit.as_ref().is_none_or(|unit| unit.is_dead())
    }

    pub fn is_pinging(&self) -> bool {
        self.ping_time > 0.0
    }

    pub fn ip(&self) -> &str {
        self.con
            .as_ref()
            .map(|con| con.address.as_str())
            .unwrap_or(LOCAL_PLAYER_IP)
    }

    pub fn uuid(&self) -> &str {
        self.con
            .as_ref()
            .map(|con| con.uuid.as_str())
            .unwrap_or(LOCAL_PLAYER_ID)
    }

    pub fn usid(&self) -> &str {
        self.con
            .as_ref()
            .map(|con| con.usid.as_str())
            .unwrap_or(LOCAL_PLAYER_ID)
    }

    pub fn colored_name(&self) -> String {
        format!("[#{:08X}]{}", self.color, self.name)
    }

    pub fn plain_name(&self) -> String {
        strip_colors(&self.name)
    }

    pub fn reset(&mut self, default_team: TeamId) {
        self.team = default_team;
        self.admin = false;
        self.typing = false;
        self.text_fade_time = 0.0;
        self.x = 0.0;
        self.y = 0.0;
        self.last_preview_plan_timestamp = 0;
        self.last_preview_plan_group = -1;
        self.last_preview_plan_group_server = -1;
        self.preview_plans_current.clear();
        self.preview_plans_assembling.clear();
        self.receiving_new_plan_group = false;
        self.preview_plans_dirty = false;

        if !self.dead() {
            self.clear_unit();
        }
    }
}

impl Default for PlayerComp {
    fn default() -> Self {
        Self::new(TeamId(TEAM_SHARDED))
    }
}

fn strip_colors(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            for next in chars.by_ref() {
                if next == ']' {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::world::block::Block;

    #[test]
    fn preview_plan_handling_commits_after_delay_and_caps_length() {
        let mut player = PlayerComp::default();
        let plans = vec![
            BuildPlan::new_place(1, 2, 0, "duo"),
            BuildPlan::new_break(3, 4),
            BuildPlan::new_place(5, 6, 1, "router"),
        ];

        player.handle_preview_plans(2, &plans, 10, 2);
        assert!(player.preview_plans_current.is_empty());
        assert_eq!(player.preview_plans_assembling.len(), 2);
        assert!(player.receiving_new_plan_group);

        let current = player.get_preview_plans(109);
        assert!(current.is_empty());
        assert!(!player.preview_plans_dirty);

        let current = player.get_preview_plans(110);
        assert_eq!(current.len(), 2);
        assert!(player.preview_plans_assembling.is_empty());
        assert!(player.preview_plans_dirty);
        assert!(!player.receiving_new_plan_group);
    }

    #[test]
    fn preview_plan_handling_ignores_outdated_groups_and_restarts_on_new_group() {
        let mut player = PlayerComp::default();
        let first = vec![BuildPlan::new_place(1, 2, 0, "duo")];
        let second = vec![BuildPlan::new_place(3, 4, 0, "router")];

        player.handle_preview_plans(1, &first, 50, 10);
        player.get_preview_plans(150);

        player.handle_preview_plans(1, &second, 160, 10);
        assert_eq!(player.preview_plans_assembling.len(), 0);

        player.handle_preview_plans(0, &second, 170, 10);
        assert_eq!(player.preview_plans_assembling.len(), 0);

        player.handle_preview_plans(2, &second, 180, 10);
        assert_eq!(player.preview_plans_assembling.len(), 1);
        assert_eq!(player.last_preview_plan_group, 2);
    }

    #[test]
    fn identity_helpers_follow_connection_or_local_fallbacks() {
        let mut player = PlayerComp::default();
        player.name = "[scarlet]frog[]".into();
        player.color = 0x11_22_33_44;

        assert_eq!(player.colored_name(), "[#11223344][scarlet]frog[]");
        assert_eq!(player.plain_name(), "frog");
        assert_eq!(player.ip(), "localhost");
        assert_eq!(player.uuid(), "[LOCAL]");
        assert_eq!(player.usid(), "[LOCAL]");

        let mut con = NetConnection::new("127.0.0.1");
        con.uuid = "uuid-1".into();
        con.usid = "usid-1".into();
        player.con = Some(con);

        assert_eq!(player.ip(), "127.0.0.1");
        assert_eq!(player.uuid(), "uuid-1");
        assert_eq!(player.usid(), "usid-1");
    }

    #[test]
    fn unit_helpers_follow_validity_build_and_ammo_flags() {
        let mut player = PlayerComp::default();

        assert!(player.dead());
        assert!(!player.is_builder());
        assert!(!player.display_ammo(false));
        assert!(!player.is_pinging());

        player.set_unit_state(
            PlayerUnitState::block(42)
                .with_can_build(true)
                .with_valid(true),
        );
        player.ping_time = 1.0;

        assert!(!player.dead());
        assert!(player.is_builder());
        assert!(player.display_ammo(false));
        assert!(player.display_ammo(true));
        assert!(player.is_pinging());
        assert_eq!(player.unit_ref(), Some(UnitRef::Block { tile_pos: 42 }));
    }

    #[test]
    fn reset_returns_to_default_team_and_clears_runtime_preview_state() {
        let mut player = PlayerComp::default();
        player.team = TeamId(7);
        player.admin = true;
        player.typing = true;
        player.x = 10.0;
        player.y = 20.0;
        player.last_preview_plan_group = 5;
        player.last_preview_plan_group_server = 6;
        player.last_preview_plan_timestamp = 777;
        player.receiving_new_plan_group = true;
        player
            .preview_plans_current
            .push(BuildPlan::new_place(1, 2, 0, "duo"));
        player
            .preview_plans_assembling
            .push(BuildPlan::new_break(3, 4));
        player.preview_plans_dirty = true;
        player.set_unit_state(PlayerUnitState::unit(9));
        player.selected_block = Some(Block::new(1, "duo"));

        player.reset(TeamId(3));

        assert_eq!(player.team, TeamId(3));
        assert!(!player.admin);
        assert!(!player.typing);
        assert_eq!((player.x, player.y), (0.0, 0.0));
        assert_eq!(player.last_preview_plan_group, -1);
        assert_eq!(player.last_preview_plan_group_server, -1);
        assert_eq!(player.last_preview_plan_timestamp, 0);
        assert!(player.preview_plans_current.is_empty());
        assert!(player.preview_plans_assembling.is_empty());
        assert!(!player.preview_plans_dirty);
        assert!(player.unit.is_none());
        assert!(player.selected_block.is_some());
    }

    #[test]
    fn core_helpers_delegate_to_callbacks_using_player_position_and_team() {
        let player = PlayerComp::new(TeamId(4));
        let core = CoreInfo::new(12, 4, 8.0, 16.0);

        assert_eq!(
            player.core_with(|team| (team == TeamId(4)).then_some(core)),
            Some(core)
        );
        assert_eq!(
            player.closest_core_with(|x, y, team| {
                (x == 0.0 && y == 0.0 && team == TeamId(4)).then_some(core)
            }),
            Some(core)
        );
    }
}
