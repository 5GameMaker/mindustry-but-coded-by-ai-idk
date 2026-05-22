use std::cmp::Ordering;

pub const RTS_SQUAD_RADIUS: f32 = 60.0;
pub const RTS_TIME_UPDATE: usize = 0;
pub const RTS_TIMER_SPAWN: usize = 1;
pub const RTS_MAX_TARGETS_CHECKED: usize = 15;
pub const RTS_DEFEND_CHECK_RANGE: f32 = 350.0;
pub const RTS_DEFEND_WITHIN_RANGE: f32 = 1_000.0;
pub const RTS_TURRET_SCAN_PADDING: f32 = 50.0;
pub const RTS_BATTLE_EPSILON: f32 = 0.001;
pub const RTS_BATTLE_YIELD_IMPOSSIBLE: f32 = 100_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum TargetPriority {
    Generator,
    Factory,
    Core,
    Battery,
    Drill,
}

pub const DEFAULT_TARGET_PRIORITIES: [TargetPriority; 5] = [
    TargetPriority::Generator,
    TargetPriority::Factory,
    TargetPriority::Core,
    TargetPriority::Battery,
    TargetPriority::Drill,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RtsAiConfig {
    pub squad_radius: f32,
    pub max_targets_checked: usize,
    pub defend_check_range: f32,
    pub defend_within_range: f32,
    pub turret_scan_padding: f32,
    pub battle_epsilon: f32,
    pub battle_yield_impossible: f32,
    pub target_priorities: [TargetPriority; 5],
}

impl Default for RtsAiConfig {
    fn default() -> Self {
        Self {
            squad_radius: RTS_SQUAD_RADIUS,
            max_targets_checked: RTS_MAX_TARGETS_CHECKED,
            defend_check_range: RTS_DEFEND_CHECK_RANGE,
            defend_within_range: RTS_DEFEND_WITHIN_RANGE,
            turret_scan_padding: RTS_TURRET_SCAN_PADDING,
            battle_epsilon: RTS_BATTLE_EPSILON,
            battle_yield_impossible: RTS_BATTLE_YIELD_IMPOSSIBLE,
            target_priorities: DEFAULT_TARGET_PRIORITIES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RtsRulesSnapshot {
    pub ai_core_spawn: bool,
    pub min_squad: usize,
    pub min_weight: f32,
    pub max_squad: usize,
    pub unit_cap: usize,
}

impl RtsRulesSnapshot {
    pub const fn new(
        ai_core_spawn: bool,
        min_squad: usize,
        min_weight: f32,
        max_squad: usize,
        unit_cap: usize,
    ) -> Self {
        Self {
            ai_core_spawn,
            min_squad,
            min_weight,
            max_squad,
            unit_cap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquadMemberSnapshot {
    pub x: f32,
    pub y: f32,
    pub health: f32,
    pub dps_estimate: f32,
    pub target_air: bool,
    pub target_ground: bool,
    pub is_naval: bool,
    pub flag: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquadSummary {
    pub size: usize,
    pub center_x: f32,
    pub center_y: f32,
    pub total_health: f32,
    pub total_dps: f32,
    pub target_air: bool,
    pub target_ground: bool,
    pub is_naval: bool,
    pub has_nonzero_flag: bool,
}

impl SquadSummary {
    pub fn from_members(members: &[SquadMemberSnapshot]) -> Option<Self> {
        let first = members.first()?;
        let mut center_x = 0.0;
        let mut center_y = 0.0;
        let mut total_health = 0.0;
        let mut total_dps = 0.0;
        let mut target_air = true;
        let mut target_ground = true;

        for member in members {
            if !member.target_air {
                target_air = false;
            }
            if !member.target_ground {
                target_ground = false;
            }

            center_x += member.x;
            center_y += member.y;
            total_health += member.health;
            total_dps += member.dps_estimate;
        }

        let size = members.len();
        let size_f = size as f32;

        Some(Self {
            size,
            center_x: center_x / size_f,
            center_y: center_y / size_f,
            total_health,
            total_dps,
            target_air,
            target_ground,
            is_naval: first.is_naval,
            has_nonzero_flag: members.iter().any(|member| member.flag != 0),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetCandidate {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub weight: f32,
    pub distance: f32,
    pub distance2: f32,
}

impl TargetCandidate {
    pub fn new(id: u32, x: f32, y: f32, origin_x: f32, origin_y: f32, weight: f32) -> Self {
        let dx = x - origin_x;
        let dy = y - origin_y;
        Self {
            id,
            x,
            y,
            weight,
            distance: (dx * dx + dy * dy).sqrt(),
            distance2: dx * dx + dy * dy,
        }
    }

    pub fn score(&self) -> f32 {
        candidate_score(self.weight, self.distance)
    }
}

pub fn candidate_score(weight: f32, distance: f32) -> f32 {
    (1.0 - weight) + distance / 10_000.0
}

pub fn select_target_candidate<'a>(
    candidates: &'a [TargetCandidate],
    check_weight: bool,
    min_weight: f32,
    squad_size: usize,
    max_squad: usize,
    unit_cap: usize,
) -> Option<&'a TargetCandidate> {
    let best = candidates.iter().min_by(|a, b| compare_candidates(a, b))?;
    if check_weight && best.weight < min_weight && squad_size < max_squad && squad_size < unit_cap {
        return None;
    }

    Some(best)
}

pub fn compare_candidates(a: &TargetCandidate, b: &TargetCandidate) -> Ordering {
    a.score()
        .partial_cmp(&b.score())
        .unwrap_or(Ordering::Equal)
        .then_with(|| {
            a.distance2
                .partial_cmp(&b.distance2)
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| a.id.cmp(&b.id))
}

pub fn point_segment_distance(
    point_x: f32,
    point_y: f32,
    segment_x1: f32,
    segment_y1: f32,
    segment_x2: f32,
    segment_y2: f32,
) -> f32 {
    let vx = segment_x2 - segment_x1;
    let vy = segment_y2 - segment_y1;
    let wx = point_x - segment_x1;
    let wy = point_y - segment_y1;

    let len2 = vx * vx + vy * vy;
    if len2 <= f32::EPSILON {
        return ((point_x - segment_x1).powi(2) + (point_y - segment_y1).powi(2)).sqrt();
    }

    let t = ((wx * vx) + (wy * vy)) / len2;
    let t = t.clamp(0.0, 1.0);
    let proj_x = segment_x1 + t * vx;
    let proj_y = segment_y1 + t * vy;
    ((point_x - proj_x).powi(2) + (point_y - proj_y).powi(2)).sqrt()
}

pub fn battle_yield(
    self_dps: f32,
    self_health: f32,
    enemy_health: f32,
    enemy_dps: f32,
    epsilon: f32,
    impossible_score: f32,
) -> f32 {
    let time_destroy_enemy = if self_dps.abs() <= epsilon {
        f32::INFINITY
    } else {
        enemy_health / self_dps
    };
    let time_destroy_self = if enemy_dps.abs() <= epsilon {
        f32::INFINITY
    } else {
        self_health / enemy_dps
    };

    if time_destroy_enemy.is_infinite() || time_destroy_self.abs() <= epsilon {
        return 0.0;
    }
    if time_destroy_self.is_infinite() || time_destroy_enemy.abs() <= epsilon {
        return impossible_score;
    }

    time_destroy_self / time_destroy_enemy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_match_upstream_constants() {
        let config = RtsAiConfig::default();
        assert_eq!(config.squad_radius, RTS_SQUAD_RADIUS);
        assert_eq!(config.max_targets_checked, RTS_MAX_TARGETS_CHECKED);
        assert_eq!(config.defend_check_range, RTS_DEFEND_CHECK_RANGE);
        assert_eq!(config.defend_within_range, RTS_DEFEND_WITHIN_RANGE);
        assert_eq!(config.turret_scan_padding, RTS_TURRET_SCAN_PADDING);
        assert_eq!(config.battle_epsilon, RTS_BATTLE_EPSILON);
        assert_eq!(config.battle_yield_impossible, RTS_BATTLE_YIELD_IMPOSSIBLE);
        assert_eq!(config.target_priorities, DEFAULT_TARGET_PRIORITIES);
    }

    #[test]
    fn squad_summary_accumulates_center_health_dps_and_flags() {
        let members = [
            SquadMemberSnapshot {
                x: 2.0,
                y: 4.0,
                health: 10.0,
                dps_estimate: 1.5,
                target_air: true,
                target_ground: true,
                is_naval: true,
                flag: 0,
            },
            SquadMemberSnapshot {
                x: 4.0,
                y: 8.0,
                health: 30.0,
                dps_estimate: 2.5,
                target_air: false,
                target_ground: true,
                is_naval: true,
                flag: 7,
            },
        ];

        let summary = SquadSummary::from_members(&members).expect("non-empty squad");
        assert_eq!(summary.size, 2);
        assert_eq!(summary.center_x, 3.0);
        assert_eq!(summary.center_y, 6.0);
        assert_eq!(summary.total_health, 40.0);
        assert_eq!(summary.total_dps, 4.0);
        assert!(!summary.target_air);
        assert!(summary.target_ground);
        assert!(summary.is_naval);
        assert!(summary.has_nonzero_flag);
    }

    #[test]
    fn point_segment_distance_handles_projections_and_endpoints() {
        let center = point_segment_distance(5.0, 3.0, 0.0, 0.0, 10.0, 0.0);
        assert!((center - 3.0).abs() < 1e-6);

        let left = point_segment_distance(-2.0, 4.0, 0.0, 0.0, 10.0, 0.0);
        assert!((left - (20.0_f32).sqrt()).abs() < 1e-6);

        let zero = point_segment_distance(2.0, 2.0, 1.0, 1.0, 1.0, 1.0);
        assert!((zero - (2.0_f32).sqrt()).abs() < 1e-6);
    }

    #[test]
    fn battle_yield_matches_java_boundary_cases() {
        assert_eq!(
            battle_yield(
                10.0,
                20.0,
                50.0,
                5.0,
                RTS_BATTLE_EPSILON,
                RTS_BATTLE_YIELD_IMPOSSIBLE
            ),
            0.8
        );
        assert_eq!(
            battle_yield(
                0.0,
                20.0,
                50.0,
                5.0,
                RTS_BATTLE_EPSILON,
                RTS_BATTLE_YIELD_IMPOSSIBLE
            ),
            0.0
        );
        assert_eq!(
            battle_yield(
                10.0,
                20.0,
                50.0,
                0.0,
                RTS_BATTLE_EPSILON,
                RTS_BATTLE_YIELD_IMPOSSIBLE
            ),
            RTS_BATTLE_YIELD_IMPOSSIBLE
        );
        assert_eq!(
            battle_yield(
                10.0,
                0.0,
                50.0,
                5.0,
                RTS_BATTLE_EPSILON,
                RTS_BATTLE_YIELD_IMPOSSIBLE
            ),
            0.0
        );
    }

    #[test]
    fn candidate_selection_orders_by_score_then_distance_and_applies_weight_gate() {
        let near_but_weak = TargetCandidate::new(1, 1.0, 0.0, 0.0, 0.0, 0.40);
        let far_but_stronger = TargetCandidate::new(2, 50.0, 0.0, 0.0, 0.0, 0.60);
        let near_and_strong = TargetCandidate::new(3, 2.0, 0.0, 0.0, 0.0, 0.80);

        assert_eq!(
            compare_candidates(&near_but_weak, &far_but_stronger),
            Ordering::Greater
        );
        assert_eq!(
            select_target_candidate(
                &[far_but_stronger, near_but_weak, near_and_strong],
                true,
                0.50,
                1,
                3,
                10,
            )
            .map(|candidate| candidate.id),
            Some(3)
        );

        assert_eq!(
            select_target_candidate(&[near_but_weak], true, 0.50, 1, 3, 10),
            None
        );

        assert_eq!(
            select_target_candidate(&[near_but_weak], true, 0.50, 3, 3, 10)
                .map(|candidate| candidate.id),
            Some(1)
        );
    }
}
