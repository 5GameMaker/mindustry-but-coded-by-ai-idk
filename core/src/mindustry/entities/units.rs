use crate::mindustry::io::{EntityRef, Point2, TeamId, TypeValue, Vec2};
use crate::mindustry::r#type::{StatusEffect, Weapon};
use crate::mindustry::world::block::Block;

/// Controller contract mirrored from upstream `mindustry.entities.units.UnitController`.
///
/// Java overloads `unit(Unit)` as setter and `unit()` as nullable getter; Rust
/// keeps the nullable entity handle as `EntityRef` and names the setter
/// explicitly to avoid overloading.
pub trait UnitController {
    fn set_unit(&mut self, unit: EntityRef);

    fn unit(&self) -> EntityRef;

    fn hit(&mut self, _bullet: EntityRef) {}

    fn is_valid_controller(&self) -> bool {
        true
    }

    /// Returns whether logic AI can take over.
    fn is_logic_controllable(&self) -> bool {
        false
    }

    fn update_unit(&mut self) {}

    fn removed(&mut self, _unit: EntityRef) {}

    fn after_read(&mut self, _unit: EntityRef) {}
}

pub const AI_ROTATE_BACK_TIMER: f32 = 60.0 * 5.0;
pub const AI_TIMER_TARGET: usize = 0;
pub const AI_TIMER_TARGET2: usize = 1;
pub const AI_TIMER_TARGET3: usize = 2;
pub const AI_TIMER_TARGET4: usize = 3;
pub const AI_TIMER_COUNT: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct AiControllerTimers {
    pub values: [f32; AI_TIMER_COUNT],
}

impl AiControllerTimers {
    pub const fn new() -> Self {
        Self {
            values: [0.0; AI_TIMER_COUNT],
        }
    }

    pub fn reset(&mut self, timer: usize, value: f32) {
        if let Some(slot) = self.values.get_mut(timer) {
            *slot = value;
        }
    }

    pub fn reset_target_timers(&mut self, target: f32, target2: f32) {
        self.reset(AI_TIMER_TARGET, target);
        self.reset(AI_TIMER_TARGET2, target2);
    }

    pub fn advance(&mut self, timer: usize, delta: f32) {
        if let Some(slot) = self.values.get_mut(timer) {
            *slot += delta;
        }
    }

    pub fn ready(&mut self, timer: usize, interval: f32) -> bool {
        if interval <= 0.0 {
            return true;
        }

        let Some(slot) = self.values.get_mut(timer) else {
            return false;
        };

        if *slot >= interval {
            *slot %= interval;
            true
        } else {
            false
        }
    }
}

impl Default for AiControllerTimers {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiController {
    unit: EntityRef,
    pub timer: AiControllerTimers,
    pub fallback: Option<EntityRef>,
    pub no_target_time: f32,
    pub target: EntityRef,
    pub bomber_target: EntityRef,
    pub turning_away: bool,
}

impl AiController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_timers(&mut self, target: f32, target2: f32) {
        self.timer.reset_target_timers(target, target2);
    }

    pub fn keep_state(&self) -> bool {
        false
    }

    pub fn fallback(&self) -> Option<EntityRef> {
        None
    }

    pub fn use_fallback(&self) -> bool {
        false
    }

    pub fn should_fire(&self) -> bool {
        true
    }

    pub fn should_shoot(&self) -> bool {
        true
    }

    pub fn init(&mut self) {}

    pub fn stop_shooting(mounts: &mut [WeaponMount]) {
        for mount in mounts {
            mount.shoot = false;
        }
    }

    pub fn target_invalidated(&mut self) {
        self.timer.reset(AI_TIMER_TARGET, -1.0);
    }

    pub fn retarget(&mut self, delta: f32, has_target: bool) -> bool {
        self.timer.advance(AI_TIMER_TARGET, delta);
        self.timer
            .ready(AI_TIMER_TARGET, if has_target { 90.0 } else { 40.0 })
    }

    pub fn update_visuals_plan(input: AiVisualInput) -> AiVisualPlan {
        if input.is_flying {
            AiVisualPlan {
                wobble: input.type_wobble,
                look_at: Some(input.pref_rotation),
            }
        } else {
            AiVisualPlan {
                wobble: false,
                look_at: None,
            }
        }
    }

    pub fn face_target_plan(input: AiFaceTargetInput) -> AiFacePlan {
        if !(input.omni_movement || input.mech) {
            return AiFacePlan { look_at: None };
        }

        if input.target_valid && input.face_target && input.has_weapons {
            if let Some(intercept) = input.intercept {
                return AiFacePlan {
                    look_at: Some(angle_to(input.unit_position, intercept)),
                };
            }
        }

        if input.moving {
            AiFacePlan {
                look_at: Some(vec_angle(input.velocity)),
            }
        } else {
            AiFacePlan { look_at: None }
        }
    }

    pub fn face_movement_plan(input: AiFaceMovementInput) -> AiFacePlan {
        if (input.omni_movement || input.mech) && input.moving {
            AiFacePlan {
                look_at: Some(vec_angle(input.velocity)),
            }
        } else {
            AiFacePlan { look_at: None }
        }
    }

    pub fn invalid(target: Option<&AiTargetSnapshot>) -> bool {
        target.map(|target| !target.valid).unwrap_or(true)
    }

    pub fn check_target(target: Option<&AiTargetSnapshot>, x: f32, y: f32, range: f32) -> bool {
        target
            .map(|target| !target.valid || !within(target.position, Vec2::new(x, y), range))
            .unwrap_or(true)
    }

    pub fn update_weapons_plan(&mut self, input: AiWeaponUpdateInput) -> AiWeaponPlan {
        if input.retarget {
            self.target = input
                .main_target
                .map(|target| target.entity)
                .unwrap_or_else(EntityRef::null);
        }

        self.no_target_time += input.delta;
        if let Some(target) = input
            .main_target
            .filter(|target| target.entity == self.target)
        {
            if !target.valid {
                self.target_invalidated();
                self.target = EntityRef::null();
            } else {
                self.no_target_time = 0.0;
            }
        } else if self.target.id.is_some() && input.single_target {
            self.target = EntityRef::null();
        }

        let mut plan = AiWeaponPlan {
            unit_aim: None,
            is_shooting: false,
            mounts: Vec::with_capacity(input.mounts.len()),
        };

        for (index, mount) in input.mounts.iter().enumerate() {
            let mount_offset = rotate_vec(
                Vec2::new(mount.weapon.x, mount.weapon.y),
                input.unit_rotation - 90.0,
            );
            let mount_position = Vec2::new(
                input.unit_position.x + mount_offset.x,
                input.unit_position.y + mount_offset.y,
            );

            if !mount.weapon.controllable || mount.weapon.no_attack {
                plan.mounts.push(AiMountPlan {
                    target: mount.target.map(|target| target.entity),
                    aim: None,
                    shoot: false,
                    rotate: false,
                    mount_position,
                    rotate_back: false,
                });
                continue;
            }

            if !mount.weapon.ai_controllable {
                plan.mounts.push(AiMountPlan {
                    target: mount.target.map(|target| target.entity),
                    aim: None,
                    shoot: false,
                    rotate: false,
                    mount_position,
                    rotate_back: false,
                });
                continue;
            }

            let mut target = if input.single_target {
                input
                    .main_target
                    .filter(|target| target.entity == self.target)
            } else if input.retarget {
                input
                    .mount_targets
                    .get(index)
                    .copied()
                    .flatten()
                    .or(mount.target)
            } else {
                mount.target
            };

            if Self::check_target(
                target.as_ref(),
                mount_position.x,
                mount_position.y,
                mount.weapon.range,
            ) {
                target = None;
            }

            let mut shoot_intent = false;
            let mut aim = None;
            if let Some(target) = target {
                shoot_intent = within(
                    target.position,
                    mount_position,
                    mount.weapon.range + target.hit_size / 2.0,
                ) && input.should_shoot;
                aim = Some(target.position);
            }

            let rotate = shoot_intent;
            let shoot = shoot_intent && input.should_fire;
            plan.is_shooting |= shoot;
            if shoot_intent {
                plan.unit_aim = aim;
            }

            let rotate_back = target.is_none()
                && !shoot_intent
                && !within_angle(mount.rotation, mount.weapon.base_rotation, 0.01)
                && self.no_target_time >= AI_ROTATE_BACK_TIMER;
            let (rotate, aim) = if rotate_back {
                let return_offset =
                    vec_from_angle(input.unit_rotation + mount.weapon.base_rotation, 5.0);
                (
                    true,
                    Some(Vec2::new(
                        mount_position.x + return_offset.x,
                        mount_position.y + return_offset.y,
                    )),
                )
            } else {
                (rotate, aim)
            };

            plan.mounts.push(AiMountPlan {
                target: target.map(|target| target.entity),
                aim,
                shoot,
                rotate,
                mount_position,
                rotate_back,
            });
        }

        plan
    }

    pub fn target_flag<'a>(
        team: TeamId,
        derelict: TeamId,
        x: f32,
        y: f32,
        targets: &'a [AiFlaggedTarget],
        flag: &str,
        enemy: bool,
    ) -> Option<&'a AiFlaggedTarget> {
        if team == derelict {
            return None;
        }

        targets
            .iter()
            .filter(|target| target.flag == flag && target.enemy == enemy)
            .min_by(|left, right| {
                dst2(Vec2::new(x, y), left.position)
                    .partial_cmp(&dst2(Vec2::new(x, y), right.position))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn target_flag_active<'a>(
        team: TeamId,
        derelict: TeamId,
        x: f32,
        y: f32,
        targets: &'a [AiFlaggedTarget],
        flag: &str,
        enemy: bool,
    ) -> Option<&'a AiFlaggedTarget> {
        if team == derelict {
            return None;
        }

        targets
            .iter()
            .filter(|target| {
                target.flag == flag
                    && target.enemy == enemy
                    && target.targetable
                    && (target.has_items || target.status != AiBlockStatus::NoInput)
            })
            .min_by(|left, right| {
                dst2(Vec2::new(x, y), left.position)
                    .partial_cmp(&dst2(Vec2::new(x, y), right.position))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn pathfind_plan(input: AiPathfindInput) -> Option<Vec2> {
        if (input.tile == input.target_tile && input.stop_at_target_tile) || !input.can_pass_target
        {
            return None;
        }

        Some(vec_from_angle(
            angle_to(input.tile_world, input.target_tile_world),
            input.pref_speed,
        ))
    }

    pub fn alter_pathfind(vec: Vec2) -> Vec2 {
        vec
    }

    pub fn unload_payloads_plan(input: AiUnloadPayloadInput) -> bool {
        input.has_payload
            && input.target_is_building
            && input.last_payload_is_unit
            && input.target_within_drop_range
    }

    pub fn circle_plan(target: Option<Vec2>, input: AiCircleInput) -> Option<Vec2> {
        let target = target?;
        let mut vec = Vec2::new(
            target.x - input.unit_position.x,
            target.y - input.unit_position.y,
        );
        let len = vec_len(vec);

        if len < input.circle_length && input.circle_length > 0.0 {
            vec = rotate_vec(
                vec,
                (input.circle_length - len) / input.circle_length * 180.0,
            );
        }

        Some(set_length(vec, input.speed))
    }

    pub fn circle_attack_plan(input: AiCircleAttackInput) -> Option<AiMovePlan> {
        let target = input.target?;
        let mut vec = Vec2::new(
            target.position.x - input.unit_position.x,
            target.position.y - input.unit_position.y,
        );
        let target_angle = angle_to(input.unit_position, target.position);
        let diff = angle_dist(target_angle, input.unit_rotation);
        let len = vec_len(vec);

        if target.same_collision_layer {
            let avoid_dist = target.physic_size + 30.0;
            if input.turning_away {
                return Some(AiMovePlan {
                    movement: Some(set_length(vec, input.pref_speed * -1.0)),
                    look_at: None,
                    direct_move: false,
                });
            } else if len <= avoid_dist {
                return Some(AiMovePlan {
                    movement: Some(set_length(vec, input.pref_speed * -1.0)),
                    look_at: None,
                    direct_move: false,
                });
            }
        }

        if diff > 70.0 && len < input.circle_length {
            vec = vec_from_angle(vec_angle(input.unit_velocity), len.max(1.0));
        } else if input.omni_movement {
            vec = vec_from_angle(
                move_toward_angle(vec_angle(input.unit_velocity), vec_angle(vec), 6.0),
                len.max(1.0),
            );
        }

        Some(AiMovePlan {
            movement: Some(set_length(vec, input.pref_speed)),
            look_at: None,
            direct_move: false,
        })
    }

    pub fn move_to_plan(target: Option<Vec2>, input: AiMoveToInput) -> Option<AiMovePlan> {
        let target = target?;
        let speed = input.pref_speed;
        let mut vec = Vec2::new(
            target.x - input.unit_position.x,
            target.y - input.unit_position.y,
        );
        let distance = vec_len(vec);
        let length = if input.circle_length <= 0.001 {
            1.0
        } else {
            ((distance - input.circle_length) / input.smooth).clamp(-1.0, 1.0)
        };

        vec = set_length(vec, speed * length);

        if input.arrive && length > 0.0 && input.accel.abs() > f32::EPSILON {
            let brake = Vec2::new(
                -input.velocity.x / input.accel * 2.0 + (target.x - input.unit_position.x),
                -input.velocity.y / input.accel * 2.0 + (target.y - input.unit_position.y),
            );

            if input.omni_movement || input.rotate_move_first {
                vec = limit_vec(Vec2::new(vec.x + brake.x, vec.y + brake.y), speed * length);
            } else {
                return Some(AiMovePlan {
                    movement: Some(limit_vec(brake, speed * length)),
                    look_at: None,
                    direct_move: true,
                });
            }
        }

        if length < -0.5 {
            if input.keep_distance {
                vec = rotate_vec(vec, 180.0);
            } else {
                vec = Vec2::new(0.0, 0.0);
            }
        } else if length < 0.0 {
            vec = Vec2::new(0.0, 0.0);
        }

        if let Some(offset) = input.offset {
            vec = Vec2::new(vec.x + offset.x, vec.y + offset.y);
            vec = set_length(vec, speed * length);
        }

        if invalid_vec(vec) || is_zero_vec(vec) {
            return None;
        }

        if !input.omni_movement && input.rotate_move_first {
            let angle = vec_angle(vec);
            Some(AiMovePlan {
                movement: within_angle(input.unit_rotation, angle, 3.0).then_some(vec),
                look_at: Some(angle),
                direct_move: false,
            })
        } else {
            Some(AiMovePlan {
                movement: Some(vec),
                look_at: None,
                direct_move: false,
            })
        }
    }
}

impl Default for AiController {
    fn default() -> Self {
        let mut controller = Self {
            unit: EntityRef::null(),
            timer: AiControllerTimers::default(),
            fallback: None,
            no_target_time: 0.0,
            target: EntityRef::null(),
            bomber_target: EntityRef::null(),
            turning_away: false,
        };
        controller.reset_timers(0.0, 0.0);
        controller
    }
}

impl UnitController for AiController {
    fn set_unit(&mut self, unit: EntityRef) {
        if self.unit != unit {
            self.unit = unit;
            self.init();
        }
    }

    fn unit(&self) -> EntityRef {
        self.unit
    }

    fn is_logic_controllable(&self) -> bool {
        true
    }

    fn after_read(&mut self, _unit: EntityRef) {}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiVisualInput {
    pub is_flying: bool,
    pub type_wobble: bool,
    pub pref_rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiVisualPlan {
    pub wobble: bool,
    pub look_at: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiFaceTargetInput {
    pub omni_movement: bool,
    pub mech: bool,
    pub moving: bool,
    pub face_target: bool,
    pub has_weapons: bool,
    pub target_valid: bool,
    pub unit_position: Vec2,
    pub velocity: Vec2,
    pub intercept: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiFaceMovementInput {
    pub omni_movement: bool,
    pub mech: bool,
    pub moving: bool,
    pub velocity: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiFacePlan {
    pub look_at: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiTargetSnapshot {
    pub entity: EntityRef,
    pub position: Vec2,
    pub hit_size: f32,
    pub valid: bool,
    pub added: bool,
    pub flying: bool,
    pub collision_layer: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiWeaponInfo {
    pub x: f32,
    pub y: f32,
    pub range: f32,
    pub base_rotation: f32,
    pub controllable: bool,
    pub ai_controllable: bool,
    pub no_attack: bool,
}

impl Default for AiWeaponInfo {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            range: 0.0,
            base_rotation: 0.0,
            controllable: true,
            ai_controllable: true,
            no_attack: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiMountInput {
    pub weapon: AiWeaponInfo,
    pub rotation: f32,
    pub target: Option<AiTargetSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiWeaponUpdateInput {
    pub unit_position: Vec2,
    pub unit_rotation: f32,
    pub delta: f32,
    pub single_target: bool,
    pub retarget: bool,
    pub main_target: Option<AiTargetSnapshot>,
    pub mount_targets: Vec<Option<AiTargetSnapshot>>,
    pub mounts: Vec<AiMountInput>,
    pub should_fire: bool,
    pub should_shoot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiMountPlan {
    pub target: Option<EntityRef>,
    pub aim: Option<Vec2>,
    pub shoot: bool,
    pub rotate: bool,
    pub mount_position: Vec2,
    pub rotate_back: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiWeaponPlan {
    pub unit_aim: Option<Vec2>,
    pub is_shooting: bool,
    pub mounts: Vec<AiMountPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiBlockStatus {
    NoInput,
    Active,
    NoOutput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiFlaggedTarget {
    pub entity: EntityRef,
    pub position: Vec2,
    pub flag: String,
    pub enemy: bool,
    pub has_items: bool,
    pub status: AiBlockStatus,
    pub targetable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiPathfindInput {
    pub tile: (i32, i32),
    pub tile_world: Vec2,
    pub target_tile: (i32, i32),
    pub target_tile_world: Vec2,
    pub stop_at_target_tile: bool,
    pub can_pass_target: bool,
    pub pref_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiUnloadPayloadInput {
    pub has_payload: bool,
    pub target_is_building: bool,
    pub last_payload_is_unit: bool,
    pub target_within_drop_range: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiCircleInput {
    pub unit_position: Vec2,
    pub circle_length: f32,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiCircleTarget {
    pub position: Vec2,
    pub same_collision_layer: bool,
    pub physic_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiCircleAttackInput {
    pub unit_position: Vec2,
    pub unit_rotation: f32,
    pub unit_velocity: Vec2,
    pub pref_speed: f32,
    pub circle_length: f32,
    pub omni_movement: bool,
    pub turning_away: bool,
    pub target: Option<AiCircleTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiMoveToInput {
    pub unit_position: Vec2,
    pub unit_rotation: f32,
    pub velocity: Vec2,
    pub pref_speed: f32,
    pub accel: f32,
    pub circle_length: f32,
    pub smooth: f32,
    pub keep_distance: bool,
    pub arrive: bool,
    pub offset: Option<Vec2>,
    pub omni_movement: bool,
    pub rotate_move_first: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AiMovePlan {
    pub movement: Option<Vec2>,
    pub look_at: Option<f32>,
    pub direct_move: bool,
}

pub const UNITS_CAP_INFINITY: i32 = i32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitCapRules {
    pub wave_team: TeamId,
    pub pvp: bool,
    pub campaign: bool,
    pub disable_unit_cap: bool,
    pub unit_cap_variable: bool,
    pub unit_cap: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitCapTeam {
    pub team: TeamId,
    pub ignore_unit_cap: bool,
    pub data_unit_cap: i32,
    pub type_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitCapType {
    pub use_unit_cap: bool,
    pub banned: bool,
}

pub fn units_get_cap(team: UnitCapTeam, rules: UnitCapRules) -> i32 {
    if (team.team == rules.wave_team && !rules.pvp)
        || (rules.campaign && team.team == rules.wave_team)
        || rules.disable_unit_cap
        || team.ignore_unit_cap
    {
        UNITS_CAP_INFINITY
    } else if rules.unit_cap_variable {
        (rules.unit_cap + team.data_unit_cap).max(0)
    } else {
        rules.unit_cap.max(0)
    }
}

pub fn units_get_string_cap(team: UnitCapTeam, rules: UnitCapRules) -> String {
    let cap = units_get_cap(team, rules);
    if cap >= UNITS_CAP_INFINITY - 1 {
        "∞".into()
    } else {
        cap.to_string()
    }
}

pub fn units_can_create(team: UnitCapTeam, unit_type: UnitCapType, rules: UnitCapRules) -> bool {
    !unit_type.use_unit_cap || (team.type_count < units_get_cap(team, rules) && !unit_type.banned)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitsRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UnitsRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn centered(x: f32, y: f32, size: f32) -> Self {
        Self::new(x - size / 2.0, y - size / 2.0, size, size)
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitsTargetKind {
    Unit,
    Building,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitsTargetSnapshot {
    pub entity: EntityRef,
    pub kind: UnitsTargetKind,
    pub team: TeamId,
    pub position: Vec2,
    pub hit_size: f32,
    pub valid: bool,
    pub dead: bool,
    pub targetable: bool,
    pub in_fog: bool,
    pub flying: bool,
    pub target_priority: f32,
}

impl UnitsTargetSnapshot {
    pub fn unit(entity_id: i32, team: TeamId, x: f32, y: f32) -> Self {
        Self {
            entity: EntityRef::new(entity_id),
            kind: UnitsTargetKind::Unit,
            team,
            position: Vec2::new(x, y),
            hit_size: 8.0,
            valid: true,
            dead: false,
            targetable: true,
            in_fog: false,
            flying: false,
            target_priority: 0.0,
        }
    }

    pub fn building(entity_id: i32, team: TeamId, x: f32, y: f32) -> Self {
        Self {
            kind: UnitsTargetKind::Building,
            ..Self::unit(entity_id, team, x, y)
        }
    }

    pub fn dst2(self, x: f32, y: f32) -> f32 {
        dst2(self.position, Vec2::new(x, y))
    }

    pub fn dst(self, x: f32, y: f32) -> f32 {
        self.dst2(x, y).sqrt()
    }

    pub fn check_target(self, air: bool, ground: bool) -> bool {
        match self.kind {
            UnitsTargetKind::Building => ground,
            UnitsTargetKind::Unit => (self.flying && air) || (!self.flying && ground),
            UnitsTargetKind::Other => air || ground,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitsEntityTileSnapshot {
    pub entity: EntityRef,
    pub tile_rect: UnitsRect,
    pub grounded: bool,
    pub allow_leg_step: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitLifecycleEffect {
    UnitCapKill,
    UnitEnvKill,
    UnitDespawn,
    DeathExplosion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitLifecyclePlan {
    pub unit: EntityRef,
    pub mark_dead: bool,
    pub killed: bool,
    pub destroy: bool,
    pub remove: bool,
    pub post_destroy_call: bool,
    pub removed_entity_id: Option<i32>,
    pub effect: Option<UnitLifecycleEffect>,
    pub shake: Option<f32>,
    pub sound_volume: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitLifecycleSnapshot {
    pub unit: EntityRef,
    pub x: f32,
    pub y: f32,
    pub hit_size: f32,
    pub death_shake: f32,
    pub death_sound_volume: f32,
}

pub fn units_unit_cap_death_plan(unit: Option<EntityRef>) -> Option<UnitLifecyclePlan> {
    unit.map(|unit| UnitLifecyclePlan {
        unit,
        mark_dead: true,
        killed: false,
        destroy: false,
        remove: false,
        post_destroy_call: true,
        removed_entity_id: unit.id,
        effect: Some(UnitLifecycleEffect::UnitCapKill),
        shake: None,
        sound_volume: None,
    })
}

pub fn units_unit_env_death_plan(unit: Option<EntityRef>) -> Option<UnitLifecyclePlan> {
    unit.map(|unit| UnitLifecyclePlan {
        unit,
        mark_dead: true,
        killed: false,
        destroy: false,
        remove: false,
        post_destroy_call: true,
        removed_entity_id: unit.id,
        effect: Some(UnitLifecycleEffect::UnitEnvKill),
        shake: None,
        sound_volume: None,
    })
}

pub fn units_unit_death_plan(uid: i32, unit_exists: bool) -> UnitLifecyclePlan {
    UnitLifecyclePlan {
        unit: EntityRef::new(uid),
        mark_dead: false,
        killed: unit_exists,
        destroy: false,
        remove: false,
        post_destroy_call: false,
        removed_entity_id: Some(uid),
        effect: None,
        shake: None,
        sound_volume: None,
    }
}

pub fn units_unit_destroy_plan(uid: i32, unit_exists: bool) -> UnitLifecyclePlan {
    UnitLifecyclePlan {
        destroy: unit_exists,
        ..units_unit_death_plan(uid, false)
    }
}

pub fn units_unit_despawn_plan(unit: Option<EntityRef>) -> Option<UnitLifecyclePlan> {
    unit.map(|unit| UnitLifecyclePlan {
        unit,
        mark_dead: false,
        killed: false,
        destroy: false,
        remove: true,
        post_destroy_call: false,
        removed_entity_id: None,
        effect: Some(UnitLifecycleEffect::UnitDespawn),
        shake: None,
        sound_volume: None,
    })
}

pub fn units_unit_safe_death_plan(
    unit: Option<UnitLifecycleSnapshot>,
) -> Option<UnitLifecyclePlan> {
    unit.map(|unit| UnitLifecyclePlan {
        unit: unit.unit,
        mark_dead: false,
        killed: false,
        destroy: false,
        remove: true,
        post_destroy_call: false,
        removed_entity_id: None,
        effect: Some(UnitLifecycleEffect::DeathExplosion),
        shake: Some(if unit.death_shake < 0.0 {
            unit.hit_size / 3.0
        } else {
            unit.death_shake
        }),
        sound_volume: Some(unit.death_sound_volume),
    })
}

pub fn units_can_interact(
    player_team: Option<TeamId>,
    tile_team: Option<TeamId>,
    tile_interactable: bool,
    editor: bool,
) -> bool {
    player_team.is_none() || tile_team.is_none() || tile_interactable || editor
}

pub fn units_is_hittable(target: Option<&UnitsTargetSnapshot>, air: bool, ground: bool) -> bool {
    target
        .map(|target| target.check_target(air, ground))
        .unwrap_or(false)
}

pub fn units_invalidate_target(
    target: Option<&UnitsTargetSnapshot>,
    team: TeamId,
    x: f32,
    y: f32,
    range: f32,
) -> bool {
    let Some(target) = target else {
        return true;
    };

    (range < f32::MAX / 2.0
        && !within(
            target.position,
            Vec2::new(x, y),
            range + target.hit_size / 2.0,
        ))
        || target.team == team
        || !target.valid
        || (target.kind == UnitsTargetKind::Unit && !target.targetable)
}

pub fn units_any_entities(
    entities: &[UnitsEntityTileSnapshot],
    rect: UnitsRect,
    ground: bool,
) -> bool {
    entities.iter().any(|unit| {
        (unit.grounded && !unit.allow_leg_step) == ground && unit.tile_rect.overlaps(rect)
    })
}

pub fn units_any_entities_centered(
    entities: &[UnitsEntityTileSnapshot],
    x: f32,
    y: f32,
    size: f32,
    ground: bool,
) -> bool {
    units_any_entities(entities, UnitsRect::centered(x, y, size), ground)
}

pub fn units_count<F>(units: &[UnitsTargetSnapshot], rect: UnitsRect, mut filter: F) -> usize
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
{
    units
        .iter()
        .filter(|unit| {
            filter(unit)
                && UnitsRect::new(
                    unit.position.x - unit.hit_size / 2.0,
                    unit.position.y - unit.hit_size / 2.0,
                    unit.hit_size,
                    unit.hit_size,
                )
                .overlaps(rect)
        })
        .count()
}

pub fn units_any<F>(units: &[UnitsTargetSnapshot], rect: UnitsRect, filter: F) -> bool
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
{
    units_count(units, rect, filter) > 0
}

pub fn units_find_enemy_tile<F>(
    buildings: &[UnitsTargetSnapshot],
    team: TeamId,
    derelict: TeamId,
    x: f32,
    y: f32,
    range: f32,
    mut pred: F,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
{
    if team == derelict {
        return None;
    }

    buildings
        .iter()
        .copied()
        .filter(|target| {
            target.kind == UnitsTargetKind::Building
                && target.team != team
                && target.team != derelict
                && pred(target)
                && within(
                    target.position,
                    Vec2::new(x, y),
                    range + target.hit_size / 2.0,
                )
        })
        .min_by(|left, right| {
            left.dst2(x, y)
                .partial_cmp(&right.dst2(x, y))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn units_closest_building<F>(
    buildings: &[UnitsTargetSnapshot],
    team: TeamId,
    x: f32,
    y: f32,
    range: f32,
    mut pred: F,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
{
    buildings
        .iter()
        .copied()
        .filter(|building| {
            building.kind == UnitsTargetKind::Building
                && building.team == team
                && pred(building)
                && building.dst(x, y) - building.hit_size / 2.0 <= range
        })
        .min_by(|left, right| {
            left.dst(x, y)
                .partial_cmp(&right.dst(x, y))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn units_closest_enemy<F>(
    units: &[UnitsTargetSnapshot],
    team: TeamId,
    derelict: TeamId,
    x: f32,
    y: f32,
    range: f32,
    mut predicate: F,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
{
    if team == derelict {
        return None;
    }

    let mut result = None;
    let mut cdist = 0.0;
    let mut cpriority = -99999.0;
    for unit in units.iter().copied() {
        if unit.kind != UnitsTargetKind::Unit
            || unit.dead
            || !predicate(&unit)
            || unit.team == team
            || unit.team == derelict
            || !unit.targetable
            || unit.in_fog
        {
            continue;
        }

        let distance = unit.dst2(x, y) - unit.hit_size * unit.hit_size;
        if distance < range * range
            && (result.is_none() || distance < cdist || unit.target_priority > cpriority)
            && unit.target_priority >= cpriority
        {
            result = Some(unit);
            cdist = distance;
            cpriority = unit.target_priority;
        }
    }
    result
}

pub fn units_best_enemy<F, S>(
    units: &[UnitsTargetSnapshot],
    team: TeamId,
    derelict: TeamId,
    x: f32,
    y: f32,
    range: f32,
    mut predicate: F,
    mut sort: S,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
    S: FnMut(&UnitsTargetSnapshot, f32, f32) -> f32,
{
    if team == derelict {
        return None;
    }

    let mut result = None;
    let mut cdist = 0.0;
    let mut cpriority = -99999.0;
    for unit in units.iter().copied() {
        if unit.kind != UnitsTargetKind::Unit
            || unit.dead
            || !predicate(&unit)
            || unit.team == team
            || unit.team == derelict
            || !within(unit.position, Vec2::new(x, y), range + unit.hit_size / 2.0)
            || !unit.targetable
            || unit.in_fog
        {
            continue;
        }

        let cost = sort(&unit, x, y);
        if (result.is_none() || cost < cdist || unit.target_priority > cpriority)
            && unit.target_priority >= cpriority
        {
            result = Some(unit);
            cdist = cost;
            cpriority = unit.target_priority;
        }
    }
    result
}

pub fn units_closest_target<F, B>(
    units: &[UnitsTargetSnapshot],
    buildings: &[UnitsTargetSnapshot],
    team: TeamId,
    derelict: TeamId,
    x: f32,
    y: f32,
    range: f32,
    unit_pred: F,
    tile_pred: B,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
    B: FnMut(&UnitsTargetSnapshot) -> bool,
{
    units_closest_enemy(units, team, derelict, x, y, range, unit_pred)
        .or_else(|| units_find_enemy_tile(buildings, team, derelict, x, y, range, tile_pred))
}

pub fn units_best_target<F, B, S>(
    units: &[UnitsTargetSnapshot],
    buildings: &[UnitsTargetSnapshot],
    team: TeamId,
    derelict: TeamId,
    x: f32,
    y: f32,
    range: f32,
    unit_pred: F,
    tile_pred: B,
    sort: S,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
    B: FnMut(&UnitsTargetSnapshot) -> bool,
    S: FnMut(&UnitsTargetSnapshot, f32, f32) -> f32,
{
    units_best_enemy(units, team, derelict, x, y, range, unit_pred, sort)
        .or_else(|| units_find_enemy_tile(buildings, team, derelict, x, y, range, tile_pred))
}

pub fn units_closest<F>(
    units: &[UnitsTargetSnapshot],
    team: TeamId,
    x: f32,
    y: f32,
    mut predicate: F,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
{
    units
        .iter()
        .copied()
        .filter(|unit| unit.team == team && predicate(unit))
        .min_by(|left, right| {
            left.dst2(x, y)
                .partial_cmp(&right.dst2(x, y))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn units_closest_in_range<F>(
    units: &[UnitsTargetSnapshot],
    team: TeamId,
    x: f32,
    y: f32,
    range: f32,
    mut predicate: F,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
{
    units
        .iter()
        .copied()
        .filter(|unit| {
            unit.team == team
                && unit.valid
                && predicate(unit)
                && within(unit.position, Vec2::new(x, y), range + unit.hit_size / 2.0)
        })
        .min_by(|left, right| {
            left.dst2(x, y)
                .partial_cmp(&right.dst2(x, y))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn units_closest_overlap<F>(
    units: &[UnitsTargetSnapshot],
    team: TeamId,
    rect: UnitsRect,
    mut predicate: F,
) -> Option<UnitsTargetSnapshot>
where
    F: FnMut(&UnitsTargetSnapshot) -> bool,
{
    units
        .iter()
        .copied()
        .filter(|unit| {
            unit.team == team
                && unit.valid
                && predicate(unit)
                && UnitsRect::new(
                    unit.position.x - unit.hit_size / 2.0,
                    unit.position.y - unit.hit_size / 2.0,
                    unit.hit_size,
                    unit.hit_size,
                )
                .overlaps(rect)
        })
        .min_by(|left, right| {
            left.dst2(rect.x, rect.y)
                .partial_cmp(&right.dst2(rect.x, rect.y))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitsTeamPresence {
    pub team: TeamId,
    pub unit_rects: Vec<UnitsRect>,
    pub turret_rects: Vec<UnitsRect>,
}

pub fn units_near_enemy(
    teams: &[UnitsTeamPresence],
    team: TeamId,
    derelict: TeamId,
    rect: UnitsRect,
) -> bool {
    teams.iter().any(|other| {
        other.team != team
            && other.team != derelict
            && (other
                .unit_rects
                .iter()
                .any(|candidate| candidate.overlaps(rect))
                || other
                    .turret_rects
                    .iter()
                    .any(|candidate| candidate.overlaps(rect)))
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusEntry {
    pub effect: Option<StatusEffect>,
    pub time: f32,
    /// for interval damage
    pub damage_time: f32,

    /// all of these are for the dynamic effect only!
    pub damage_multiplier: f32,
    pub health_multiplier: f32,
    pub speed_multiplier: f32,
    pub reload_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub drag_multiplier: f32,
    pub armor_override: f32,
}

impl StatusEntry {
    pub fn new(effect: StatusEffect, time: f32) -> Self {
        Self {
            effect: Some(effect),
            time,
            damage_time: 0.0,
            damage_multiplier: 1.0,
            health_multiplier: 1.0,
            speed_multiplier: 1.0,
            reload_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            drag_multiplier: 1.0,
            armor_override: -1.0,
        }
    }

    pub fn set(&mut self, effect: StatusEffect, time: f32) -> &mut Self {
        self.effect = Some(effect);
        self.time = time;
        self
    }
}

impl Default for StatusEntry {
    fn default() -> Self {
        Self {
            effect: None,
            time: 0.0,
            damage_time: 0.0,
            damage_multiplier: 1.0,
            health_multiplier: 1.0,
            speed_multiplier: 1.0,
            reload_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            drag_multiplier: 1.0,
            armor_override: -1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponMount {
    /// weapon associated with this mount
    pub weapon: Weapon,
    /// reload in frames; 0 means ready to fire
    pub reload: f32,
    /// rotation relative to the unit this mount is on
    pub rotation: f32,
    /// weapon recoil
    pub recoil: f32,
    /// weapon barrel recoil
    pub recoils: Option<Vec<f32>>,
    /// destination rotation; do not modify!
    pub target_rotation: f32,
    /// current heat, 0 to 1
    pub heat: f32,
    /// lerps to 1 when shooting, 0 when not
    pub warmup: f32,
    /// is the weapon actively charging
    pub charging: bool,
    /// counts up to 1 when charging, 0 when not
    pub charge: f32,
    /// lerps to reload time
    pub smooth_reload: f32,
    /// aiming position in world coordinates
    pub aim_x: f32,
    pub aim_y: f32,
    /// whether to shoot right now
    pub shoot: bool,
    /// whether to allow any shooting effects
    pub allow_shoot_effects: bool,
    /// whether to rotate to face the target right now
    pub rotate: bool,
    /// extra state for alternating weapons
    pub side: bool,
    /// total bullets fired from this mount
    pub total_shots: i32,
    /// counter for which barrel bullets have been fired from; used for alternating patterns
    pub barrel_counter: i32,
    /// Last aim length of weapon. Only used for point lasers.
    pub last_length: f32,
    /// current bullet for continuous weapons
    pub bullet: Option<String>,
    /// sound loop for continuous weapons
    pub sound: Option<String>,
    /// current target; used for autonomous weapons and AI
    pub target: Option<String>,
    /// retarget counter
    pub retarget: f32,
}

impl WeaponMount {
    pub fn new(weapon: Weapon) -> Self {
        let rotation = weapon.base_rotation;
        Self {
            weapon,
            reload: 0.0,
            rotation,
            recoil: 0.0,
            recoils: None,
            target_rotation: rotation,
            heat: 0.0,
            warmup: 0.0,
            charging: false,
            charge: 0.0,
            smooth_reload: 0.0,
            aim_x: 0.0,
            aim_y: 0.0,
            shoot: false,
            allow_shoot_effects: true,
            rotate: false,
            side: false,
            total_shots: 0,
            barrel_counter: 0,
            last_length: 0.0,
            bullet: None,
            sound: None,
            target: None,
            retarget: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildPlan {
    /// Position and rotation of this plan.
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    /// Block being placed. If null, this is a breaking plan.
    pub block: Option<String>,
    /// Whether this is a break plan.
    pub breaking: bool,
    /// Config value, matching Java `BuildPlan.config` / `TypeIO.writeObject`.
    pub config: TypeValue,

    /// Last progress.
    pub progress: f32,
    /// Whether construction has started for this plan.
    pub initialized: bool,
    pub stuck: bool,
    pub cached_valid: bool,
    /// If true, this plan is in the world. If false, it is being rendered in a schematic.
    pub world_context: bool,

    /// Visual scale. Used only for rendering.
    pub anim_scale: f32,
}

impl BuildPlan {
    pub fn new_place(x: i32, y: i32, rotation: i32, block: impl Into<String>) -> Self {
        Self {
            x,
            y,
            rotation,
            block: Some(block.into()),
            breaking: false,
            config: TypeValue::Null,
            progress: 0.0,
            initialized: false,
            stuck: false,
            cached_valid: false,
            world_context: true,
            anim_scale: 0.0,
        }
    }

    pub fn new_place_block(x: i32, y: i32, rotation: i32, block: &Block) -> Self {
        Self::new_place(x, y, block.plan_rotation(rotation), block.name.clone())
    }

    pub fn new_config(
        x: i32,
        y: i32,
        rotation: i32,
        block: impl Into<String>,
        config: TypeValue,
    ) -> Self {
        Self {
            config,
            ..Self::new_place(x, y, rotation, block)
        }
    }

    pub fn new_config_block(
        x: i32,
        y: i32,
        rotation: i32,
        block: &Block,
        config: TypeValue,
    ) -> Self {
        Self {
            config,
            ..Self::new_place_block(x, y, rotation, block)
        }
    }

    pub fn new_string_config(
        x: i32,
        y: i32,
        rotation: i32,
        block: impl Into<String>,
        config: impl Into<String>,
    ) -> Self {
        Self::new_config(x, y, rotation, block, TypeValue::String(config.into()))
    }

    pub fn new_break(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            rotation: -1,
            block: None,
            breaking: true,
            config: TypeValue::Null,
            progress: 0.0,
            initialized: false,
            stuck: false,
            cached_valid: false,
            world_context: true,
            anim_scale: 0.0,
        }
    }

    pub fn same_pos(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn set_place(&mut self, x: i32, y: i32, rotation: i32, block: impl Into<String>) {
        self.x = x;
        self.y = y;
        self.rotation = rotation;
        self.block = Some(block.into());
        self.breaking = false;
    }

    pub fn set_place_block(&mut self, x: i32, y: i32, rotation: i32, block: &Block) {
        self.set_place(x, y, block.plan_rotation(rotation), block.name.clone());
    }

    pub fn set_break(&mut self) {
        self.rotation = -1;
        self.block = None;
        self.breaking = true;
    }

    pub fn point_config_value<F>(config: &TypeValue, mut transform: F) -> TypeValue
    where
        F: FnMut(Point2) -> Point2,
    {
        match config {
            TypeValue::Point2(point) => TypeValue::Point2(transform(*point)),
            TypeValue::Point2Array(points) => {
                TypeValue::Point2Array(points.iter().copied().map(transform).collect())
            }
            _ => config.clone(),
        }
    }

    pub fn point_config<F>(&mut self, transform: F)
    where
        F: FnMut(Point2) -> Point2,
    {
        self.config = Self::point_config_value(&self.config, transform);
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }
}

impl Default for BuildPlan {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            rotation: 0,
            block: None,
            breaking: false,
            config: TypeValue::Null,
            progress: 0.0,
            initialized: false,
            stuck: false,
            cached_valid: false,
            world_context: true,
            anim_scale: 0.0,
        }
    }
}

fn dst2(left: Vec2, right: Vec2) -> f32 {
    let dx = right.x - left.x;
    let dy = right.y - left.y;
    dx * dx + dy * dy
}

fn within(left: Vec2, right: Vec2, range: f32) -> bool {
    dst2(left, right) <= range * range
}

fn vec_len(vec: Vec2) -> f32 {
    (vec.x * vec.x + vec.y * vec.y).sqrt()
}

fn is_zero_vec(vec: Vec2) -> bool {
    vec.x.abs() <= f32::EPSILON && vec.y.abs() <= f32::EPSILON
}

fn invalid_vec(vec: Vec2) -> bool {
    !vec.x.is_finite() || !vec.y.is_finite()
}

fn vec_angle(vec: Vec2) -> f32 {
    vec.y.atan2(vec.x).to_degrees().rem_euclid(360.0)
}

fn angle_to(from: Vec2, to: Vec2) -> f32 {
    vec_angle(Vec2::new(to.x - from.x, to.y - from.y))
}

fn vec_from_angle(angle: f32, length: f32) -> Vec2 {
    let rad = angle.to_radians();
    Vec2::new(rad.cos() * length, rad.sin() * length)
}

fn set_length(vec: Vec2, length: f32) -> Vec2 {
    let current = vec_len(vec);
    if current <= f32::EPSILON {
        Vec2::new(0.0, 0.0)
    } else {
        let scale = length / current;
        Vec2::new(vec.x * scale, vec.y * scale)
    }
}

fn limit_vec(vec: Vec2, limit: f32) -> Vec2 {
    let len = vec_len(vec);
    if len > limit.abs() && len > f32::EPSILON {
        set_length(vec, limit)
    } else {
        vec
    }
}

fn rotate_vec(vec: Vec2, degrees: f32) -> Vec2 {
    let rad = degrees.to_radians();
    Vec2::new(
        vec.x * rad.cos() - vec.y * rad.sin(),
        vec.x * rad.sin() + vec.y * rad.cos(),
    )
}

fn angle_dist(a: f32, b: f32) -> f32 {
    let diff = (a - b).rem_euclid(360.0).abs();
    diff.min(360.0 - diff)
}

fn within_angle(a: f32, b: f32, margin: f32) -> bool {
    angle_dist(a, b) <= margin
}

fn move_toward_angle(from: f32, to: f32, step: f32) -> f32 {
    let delta = ((to - from + 540.0).rem_euclid(360.0)) - 180.0;
    if delta.abs() <= step {
        to.rem_euclid(360.0)
    } else {
        (from + step * delta.signum()).rem_euclid(360.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::mindustry::io::{EntityRef, Point2, TeamId, TypeValue, Vec2};
    use crate::mindustry::r#type::{StatusEffect, Weapon};
    use crate::mindustry::world::block::Block;

    use super::{
        units_any, units_any_entities_centered, units_best_enemy, units_can_create,
        units_can_interact, units_closest_building, units_closest_enemy, units_closest_in_range,
        units_closest_target, units_count, units_get_cap, units_get_string_cap,
        units_invalidate_target, units_is_hittable, units_near_enemy, units_unit_cap_death_plan,
        units_unit_death_plan, units_unit_safe_death_plan, AiBlockStatus, AiCircleInput,
        AiController, AiFaceMovementInput, AiFaceTargetInput, AiFlaggedTarget, AiMountInput,
        AiMoveToInput, AiPathfindInput, AiTargetSnapshot, AiUnloadPayloadInput, AiVisualInput,
        AiWeaponInfo, AiWeaponUpdateInput, BuildPlan, StatusEntry, UnitCapRules, UnitCapTeam,
        UnitCapType, UnitController, UnitLifecycleEffect, UnitLifecycleSnapshot,
        UnitsEntityTileSnapshot, UnitsRect, UnitsTargetSnapshot, UnitsTeamPresence, WeaponMount,
        AI_ROTATE_BACK_TIMER, AI_TIMER_TARGET, UNITS_CAP_INFINITY,
    };

    #[derive(Debug)]
    struct MockUnitController {
        unit: EntityRef,
        hits: Vec<EntityRef>,
        updates: usize,
        removed: Vec<EntityRef>,
        after_reads: Vec<EntityRef>,
    }

    impl Default for MockUnitController {
        fn default() -> Self {
            Self {
                unit: EntityRef::null(),
                hits: Vec::new(),
                updates: 0,
                removed: Vec::new(),
                after_reads: Vec::new(),
            }
        }
    }

    impl UnitController for MockUnitController {
        fn set_unit(&mut self, unit: EntityRef) {
            self.unit = unit;
        }

        fn unit(&self) -> EntityRef {
            self.unit
        }

        fn hit(&mut self, bullet: EntityRef) {
            self.hits.push(bullet);
        }

        fn update_unit(&mut self) {
            self.updates += 1;
        }

        fn removed(&mut self, unit: EntityRef) {
            self.removed.push(unit);
        }

        fn after_read(&mut self, unit: EntityRef) {
            self.after_reads.push(unit);
        }
    }

    #[derive(Debug)]
    struct DefaultOnlyController {
        unit: EntityRef,
    }

    impl Default for DefaultOnlyController {
        fn default() -> Self {
            Self {
                unit: EntityRef::null(),
            }
        }
    }

    impl UnitController for DefaultOnlyController {
        fn set_unit(&mut self, unit: EntityRef) {
            self.unit = unit;
        }

        fn unit(&self) -> EntityRef {
            self.unit
        }
    }

    #[test]
    fn unit_controller_required_unit_accessors_and_defaults_match_java_interface() {
        let mut controller = DefaultOnlyController::default();
        assert_eq!(controller.unit(), EntityRef::null());

        controller.set_unit(EntityRef::new(42));
        assert_eq!(controller.unit(), EntityRef::new(42));
        assert!(controller.is_valid_controller());
        assert!(!controller.is_logic_controllable());

        controller.hit(EntityRef::new(7));
        controller.update_unit();
        controller.removed(EntityRef::new(42));
        controller.after_read(EntityRef::new(42));
        assert_eq!(controller.unit(), EntityRef::new(42));
    }

    #[test]
    fn unit_controller_hooks_can_be_overridden_by_runtime_controllers() {
        let mut controller = MockUnitController::default();
        controller.set_unit(EntityRef::new(3));
        controller.hit(EntityRef::new(9));
        controller.update_unit();
        controller.removed(EntityRef::new(3));
        controller.after_read(EntityRef::new(4));

        assert_eq!(controller.unit(), EntityRef::new(3));
        assert_eq!(controller.hits, vec![EntityRef::new(9)]);
        assert_eq!(controller.updates, 1);
        assert_eq!(controller.removed, vec![EntityRef::new(3)]);
        assert_eq!(controller.after_reads, vec![EntityRef::new(4)]);
    }

    #[test]
    fn ai_controller_defaults_timers_and_unit_assignment_match_java_shell() {
        let mut controller = AiController::new();
        assert_eq!(controller.unit(), EntityRef::null());
        assert!(controller.is_logic_controllable());
        assert!(!controller.keep_state());
        assert!(!controller.use_fallback());
        assert_eq!(controller.fallback(), None);
        assert!(controller.should_fire());
        assert!(controller.should_shoot());

        controller.set_unit(EntityRef::new(12));
        controller.set_unit(EntityRef::new(12));
        assert_eq!(controller.unit(), EntityRef::new(12));

        controller.reset_timers(39.0, 11.0);
        assert!(!controller.retarget(0.5, false));
        assert!(controller.retarget(0.5, false));
        assert!(!controller.retarget(1.0, true));

        controller.target_invalidated();
        assert_eq!(controller.timer.values[AI_TIMER_TARGET], -1.0);

        let mut weapon = Weapon::new("duo");
        weapon.base_rotation = 15.0;
        let mut mounts = vec![WeaponMount::new(weapon)];
        mounts[0].shoot = true;
        AiController::stop_shooting(&mut mounts);
        assert!(!mounts[0].shoot);
    }

    #[test]
    fn ai_controller_visual_and_facing_plans_follow_java_branches() {
        let visual = AiController::update_visuals_plan(AiVisualInput {
            is_flying: true,
            type_wobble: true,
            pref_rotation: 135.0,
        });
        assert!(visual.wobble);
        assert_eq!(visual.look_at, Some(135.0));

        let grounded = AiController::update_visuals_plan(AiVisualInput {
            is_flying: false,
            type_wobble: true,
            pref_rotation: 90.0,
        });
        assert_eq!(grounded.look_at, None);
        assert!(!grounded.wobble);

        let face_target = AiController::face_target_plan(AiFaceTargetInput {
            omni_movement: true,
            mech: false,
            moving: true,
            face_target: true,
            has_weapons: true,
            target_valid: true,
            unit_position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 1.0),
            intercept: Some(Vec2::new(10.0, 0.0)),
        });
        assert_eq!(face_target.look_at, Some(0.0));

        let face_movement = AiController::face_movement_plan(AiFaceMovementInput {
            omni_movement: false,
            mech: true,
            moving: true,
            velocity: Vec2::new(0.0, -3.0),
        });
        assert_eq!(face_movement.look_at, Some(270.0));
    }

    #[test]
    fn ai_controller_target_flag_pathfind_payload_and_movement_are_pure_plans() {
        let targets = vec![
            AiFlaggedTarget {
                entity: EntityRef::new(1),
                position: Vec2::new(5.0, 0.0),
                flag: "core".into(),
                enemy: true,
                has_items: false,
                status: AiBlockStatus::NoInput,
                targetable: true,
            },
            AiFlaggedTarget {
                entity: EntityRef::new(2),
                position: Vec2::new(8.0, 0.0),
                flag: "core".into(),
                enemy: true,
                has_items: true,
                status: AiBlockStatus::Active,
                targetable: true,
            },
        ];
        let derelict = TeamId(255);
        let closest =
            AiController::target_flag(TeamId(1), derelict, 0.0, 0.0, &targets, "core", true)
                .unwrap();
        assert_eq!(closest.entity, EntityRef::new(1));
        let active =
            AiController::target_flag_active(TeamId(1), derelict, 0.0, 0.0, &targets, "core", true)
                .unwrap();
        assert_eq!(active.entity, EntityRef::new(2));
        assert!(
            AiController::target_flag(derelict, derelict, 0.0, 0.0, &targets, "core", true)
                .is_none()
        );

        let invalid = AiTargetSnapshot {
            entity: EntityRef::new(5),
            position: Vec2::new(100.0, 0.0),
            hit_size: 4.0,
            valid: false,
            added: false,
            flying: false,
            collision_layer: 0,
        };
        assert!(AiController::invalid(Some(&invalid)));
        assert!(AiController::check_target(Some(&invalid), 0.0, 0.0, 10.0));

        let path = AiController::pathfind_plan(AiPathfindInput {
            tile: (0, 0),
            tile_world: Vec2::new(0.0, 0.0),
            target_tile: (1, 0),
            target_tile_world: Vec2::new(8.0, 0.0),
            stop_at_target_tile: true,
            can_pass_target: true,
            pref_speed: 2.0,
        })
        .unwrap();
        assert_eq!(path, Vec2::new(2.0, 0.0));

        assert!(AiController::pathfind_plan(AiPathfindInput {
            tile: (1, 0),
            tile_world: Vec2::new(8.0, 0.0),
            target_tile: (1, 0),
            target_tile_world: Vec2::new(8.0, 0.0),
            stop_at_target_tile: true,
            can_pass_target: true,
            pref_speed: 2.0,
        })
        .is_none());

        assert!(AiController::unload_payloads_plan(AiUnloadPayloadInput {
            has_payload: true,
            target_is_building: true,
            last_payload_is_unit: true,
            target_within_drop_range: true,
        }));

        let circle = AiController::circle_plan(
            Some(Vec2::new(10.0, 0.0)),
            AiCircleInput {
                unit_position: Vec2::new(0.0, 0.0),
                circle_length: 20.0,
                speed: 4.0,
            },
        )
        .unwrap();
        assert!((circle.x - 0.0).abs() < 0.0001);
        assert!((circle.y - 4.0).abs() < 0.0001);

        let move_plan = AiController::move_to_plan(
            Some(Vec2::new(100.0, 0.0)),
            AiMoveToInput {
                unit_position: Vec2::new(0.0, 0.0),
                unit_rotation: 90.0,
                velocity: Vec2::new(0.0, 0.0),
                pref_speed: 5.0,
                accel: 1.0,
                circle_length: 10.0,
                smooth: 100.0,
                keep_distance: false,
                arrive: false,
                offset: None,
                omni_movement: false,
                rotate_move_first: true,
            },
        )
        .unwrap();
        assert_eq!(move_plan.look_at, Some(0.0));
        assert_eq!(move_plan.movement, None);
    }

    #[test]
    fn ai_controller_weapon_plan_handles_single_target_fire_gate_and_rotate_back() {
        let target = AiTargetSnapshot {
            entity: EntityRef::new(77),
            position: Vec2::new(10.0, 0.0),
            hit_size: 4.0,
            valid: true,
            added: true,
            flying: false,
            collision_layer: 0,
        };
        let weapon = AiWeaponInfo {
            range: 20.0,
            base_rotation: 0.0,
            controllable: true,
            ai_controllable: true,
            no_attack: false,
            ..Default::default()
        };
        let mut controller = AiController::new();
        let plan = controller.update_weapons_plan(AiWeaponUpdateInput {
            unit_position: Vec2::new(0.0, 0.0),
            unit_rotation: 90.0,
            delta: 1.0,
            single_target: true,
            retarget: true,
            main_target: Some(target),
            mount_targets: Vec::new(),
            mounts: vec![AiMountInput {
                weapon,
                rotation: 0.0,
                target: None,
            }],
            should_fire: false,
            should_shoot: true,
        });

        assert_eq!(controller.target, EntityRef::new(77));
        assert_eq!(plan.unit_aim, Some(Vec2::new(10.0, 0.0)));
        assert!(!plan.is_shooting);
        assert_eq!(plan.mounts[0].target, Some(EntityRef::new(77)));
        assert!(!plan.mounts[0].shoot);
        assert!(plan.mounts[0].rotate);

        controller.target = EntityRef::null();
        controller.no_target_time = AI_ROTATE_BACK_TIMER;
        let rotate_back = controller.update_weapons_plan(AiWeaponUpdateInput {
            unit_position: Vec2::new(0.0, 0.0),
            unit_rotation: 90.0,
            delta: 0.0,
            single_target: false,
            retarget: false,
            main_target: None,
            mount_targets: Vec::new(),
            mounts: vec![AiMountInput {
                weapon,
                rotation: 45.0,
                target: None,
            }],
            should_fire: true,
            should_shoot: true,
        });
        assert!(rotate_back.mounts[0].rotate_back);
        assert!(rotate_back.mounts[0].rotate);
        let aim = rotate_back.mounts[0].aim.unwrap();
        assert!(aim.x.abs() < 0.0001);
        assert!((aim.y - 5.0).abs() < 0.0001);
    }

    #[test]
    fn units_cap_create_and_lifecycle_plans_follow_upstream_branches() {
        let rules = UnitCapRules {
            wave_team: TeamId(2),
            pvp: false,
            campaign: false,
            disable_unit_cap: false,
            unit_cap_variable: true,
            unit_cap: 10,
        };
        let team = UnitCapTeam {
            team: TeamId(1),
            ignore_unit_cap: false,
            data_unit_cap: 3,
            type_count: 12,
        };
        assert_eq!(units_get_cap(team, rules), 13);
        assert_eq!(units_get_string_cap(team, rules), "13");
        assert!(units_can_create(
            team,
            UnitCapType {
                use_unit_cap: true,
                banned: false,
            },
            rules
        ));

        let capped = UnitCapTeam {
            type_count: 13,
            ..team
        };
        assert!(!units_can_create(
            capped,
            UnitCapType {
                use_unit_cap: true,
                banned: false,
            },
            rules
        ));

        let wave = UnitCapTeam {
            team: TeamId(2),
            ..team
        };
        assert_eq!(units_get_cap(wave, rules), UNITS_CAP_INFINITY);
        assert_eq!(units_get_string_cap(wave, rules), "∞");

        let cap_death = units_unit_cap_death_plan(Some(EntityRef::new(5))).unwrap();
        assert!(cap_death.mark_dead);
        assert!(cap_death.post_destroy_call);
        assert_eq!(cap_death.effect, Some(UnitLifecycleEffect::UnitCapKill));

        let death = units_unit_death_plan(6, true);
        assert_eq!(death.removed_entity_id, Some(6));
        assert!(death.killed);

        let safe = units_unit_safe_death_plan(Some(UnitLifecycleSnapshot {
            unit: EntityRef::new(7),
            x: 10.0,
            y: 20.0,
            hit_size: 30.0,
            death_shake: -1.0,
            death_sound_volume: 0.6,
        }))
        .unwrap();
        assert!(safe.remove);
        assert_eq!(safe.effect, Some(UnitLifecycleEffect::DeathExplosion));
        assert_eq!(safe.shake, Some(10.0));
        assert_eq!(safe.sound_volume, Some(0.6));
    }

    #[test]
    fn units_target_validation_hittable_and_entity_rect_checks_are_pure() {
        let team = TeamId(1);
        let enemy = TeamId(2);
        let mut flying = UnitsTargetSnapshot::unit(1, enemy, 10.0, 0.0);
        flying.flying = true;
        flying.hit_size = 4.0;
        assert!(units_is_hittable(Some(&flying), true, false));
        assert!(!units_is_hittable(Some(&flying), false, true));
        assert!(!units_invalidate_target(Some(&flying), team, 0.0, 0.0, 8.0));
        assert!(units_invalidate_target(Some(&flying), team, 0.0, 0.0, 1.0));

        let same_team = UnitsTargetSnapshot::unit(2, team, 0.0, 0.0);
        assert!(units_invalidate_target(
            Some(&same_team),
            team,
            0.0,
            0.0,
            f32::MAX
        ));
        assert!(units_invalidate_target(None, team, 0.0, 0.0, f32::MAX));

        assert!(units_can_interact(Some(team), Some(enemy), true, false));
        assert!(units_can_interact(None, Some(enemy), false, false));
        assert!(units_can_interact(Some(team), Some(enemy), false, true));
        assert!(!units_can_interact(Some(team), Some(enemy), false, false));

        let entities = vec![
            UnitsEntityTileSnapshot {
                entity: EntityRef::new(1),
                tile_rect: UnitsRect::new(-2.0, -2.0, 4.0, 4.0),
                grounded: true,
                allow_leg_step: false,
            },
            UnitsEntityTileSnapshot {
                entity: EntityRef::new(2),
                tile_rect: UnitsRect::new(20.0, 20.0, 4.0, 4.0),
                grounded: true,
                allow_leg_step: true,
            },
        ];
        assert!(units_any_entities_centered(&entities, 0.0, 0.0, 8.0, true));
        assert!(!units_any_entities_centered(
            &entities, 0.0, 0.0, 8.0, false
        ));
    }

    #[test]
    fn units_selection_helpers_match_enemy_priority_and_fallback_order() {
        let team = TeamId(1);
        let enemy = TeamId(2);
        let derelict = TeamId(255);
        let mut close = UnitsTargetSnapshot::unit(1, enemy, 5.0, 0.0);
        close.hit_size = 1.0;
        close.target_priority = 0.0;
        let mut far_priority = UnitsTargetSnapshot::unit(2, enemy, 20.0, 0.0);
        far_priority.hit_size = 1.0;
        far_priority.target_priority = 10.0;
        let ally = UnitsTargetSnapshot::unit(3, team, 1.0, 0.0);
        let units = vec![close, far_priority, ally];

        let selected =
            units_closest_enemy(&units, team, derelict, 0.0, 0.0, 30.0, |_| true).unwrap();
        assert_eq!(selected.entity, EntityRef::new(2));

        let best = units_best_enemy(
            &units,
            team,
            derelict,
            0.0,
            0.0,
            30.0,
            |_| true,
            |unit, _, _| {
                if unit.entity == EntityRef::new(1) {
                    0.0
                } else {
                    100.0
                }
            },
        )
        .unwrap();
        assert_eq!(best.entity, EntityRef::new(2));

        let buildings = vec![UnitsTargetSnapshot::building(10, enemy, 3.0, 0.0)];
        let no_units = units_closest_target(
            &[],
            &buildings,
            team,
            derelict,
            0.0,
            0.0,
            10.0,
            |_| true,
            |_| true,
        )
        .unwrap();
        assert_eq!(no_units.entity, EntityRef::new(10));

        let closest_ally = units_closest_in_range(&units, team, 0.0, 0.0, 5.0, |_| true).unwrap();
        assert_eq!(closest_ally.entity, EntityRef::new(3));

        let building = units_closest_building(&buildings, enemy, 0.0, 0.0, 10.0, |_| true).unwrap();
        assert_eq!(building.entity, EntityRef::new(10));

        let rect = UnitsRect::new(-1.0, -1.0, 8.0, 8.0);
        assert_eq!(units_count(&units, rect, |_| true), 2);
        assert!(units_any(&units, rect, |unit| unit.team == enemy));

        let present = vec![UnitsTeamPresence {
            team: enemy,
            unit_rects: vec![UnitsRect::new(2.0, 2.0, 4.0, 4.0)],
            turret_rects: Vec::new(),
        }];
        assert!(units_near_enemy(&present, team, derelict, rect));
    }

    #[test]
    fn status_entry_set_attaches_effect_and_time() {
        let effect = StatusEffect::new(1, "burning");
        let mut entry = StatusEntry::default();

        entry.set(effect.clone(), 30.0);

        assert_eq!(entry.effect, Some(effect));
        assert_eq!(entry.time, 30.0);
        assert_eq!(entry.damage_multiplier, 1.0);
    }

    #[test]
    fn weapon_mount_uses_weapon_base_rotation() {
        let mut weapon = Weapon::new("duo");
        weapon.base_rotation = 45.0;

        let mount = WeaponMount::new(weapon.clone());

        assert_eq!(mount.weapon, weapon);
        assert_eq!(mount.rotation, 45.0);
        assert_eq!(mount.target_rotation, 45.0);
    }

    #[test]
    fn build_plan_supports_place_break_and_copy() {
        let mut plan = BuildPlan::new_place(3, 4, 1, "duo".to_string());
        assert_eq!(plan.block.as_deref(), Some("duo"));
        assert!(!plan.breaking);
        assert!(plan.same_pos(&BuildPlan::new_break(3, 4)));
        assert_eq!(plan.config, TypeValue::Null);

        let configured =
            BuildPlan::new_config(5, 6, 2, "router", TypeValue::Point2(Point2::new(1, 2)));
        assert_eq!(configured.config, TypeValue::Point2(Point2::new(1, 2)));

        let string_configured = BuildPlan::new_string_config(5, 6, 2, "router", "alpha");
        assert_eq!(string_configured.config, TypeValue::String("alpha".into()));

        plan.set_break();
        assert!(plan.breaking);
        assert_eq!(plan.rotation, -1);
        assert_eq!(plan.block, None);

        let copy = plan.copy();
        assert_eq!(copy, plan);
    }

    #[test]
    fn build_plan_point_config_transforms_point_values_without_losing_type() {
        let mut plan =
            BuildPlan::new_config(10, 20, 0, "router", TypeValue::Point2(Point2::new(1, 2)));

        plan.point_config(|point| Point2::new(point.x + 10, point.y - 1));

        assert_eq!(plan.config, TypeValue::Point2(Point2::new(11, 1)));

        plan.config = TypeValue::Point2Array(vec![Point2::new(0, 0), Point2::new(2, 3)]);
        plan.point_config(|point| Point2::new(point.x * 2, point.y * 3));

        assert_eq!(
            plan.config,
            TypeValue::Point2Array(vec![Point2::new(0, 0), Point2::new(4, 9)])
        );

        let string_config = TypeValue::String("unchanged".into());
        assert_eq!(
            BuildPlan::point_config_value(&string_config, |point| {
                Point2::new(point.x + 1, point.y + 1)
            }),
            string_config
        );
    }

    #[test]
    fn build_plan_block_helpers_apply_block_plan_rotation() {
        let mut block = Block::new(5, "sorter");

        let locked = BuildPlan::new_place_block(1, 2, 3, &block);
        assert_eq!(locked.block.as_deref(), Some("sorter"));
        assert_eq!(locked.rotation, 0);

        block.rotate = true;
        let rotating =
            BuildPlan::new_config_block(3, 4, 5, &block, TypeValue::String("cfg".into()));
        assert_eq!(rotating.rotation, 1);
        assert_eq!(rotating.config, TypeValue::String("cfg".into()));

        block.rotate = false;
        block.lock_rotation = false;
        let mut plan = BuildPlan::new_break(0, 0);
        plan.set_place_block(7, 8, -1, &block);
        assert_eq!((plan.x, plan.y, plan.rotation), (7, 8, 3));
        assert_eq!(plan.block.as_deref(), Some("sorter"));
        assert!(!plan.breaking);
    }
}
