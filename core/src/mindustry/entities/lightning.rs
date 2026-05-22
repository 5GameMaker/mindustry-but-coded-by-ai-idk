use std::collections::HashSet;

use crate::mindustry::{core::World, vars::TILE_SIZE};

pub const MAX_CHAIN: usize = 8;
pub const HIT_RANGE: f32 = 30.0;
const JITTER_RANGE: f32 = 3.0;
const ANGLE_JITTER: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightningPoint {
    pub x: f32,
    pub y: f32,
}

impl LightningPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightningTarget {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub enemy: bool,
    pub hittable_air: bool,
    pub hittable_ground: bool,
}

impl LightningTarget {
    pub const fn new(id: i32, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            enemy: true,
            hittable_air: true,
            hittable_ground: true,
        }
    }

    pub const fn allied(mut self) -> Self {
        self.enemy = false;
        self
    }

    pub const fn target_flags(mut self, air: bool, ground: bool) -> Self {
        self.hittable_air = air;
        self.hittable_ground = ground;
        self
    }

    pub fn check_target(&self, collides_air: bool, collides_ground: bool) -> bool {
        (collides_air && self.hittable_air) || (collides_ground && self.hittable_ground)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightningHitter {
    pub damage_multiplier: f32,
    pub collides_air: bool,
    pub collides_ground: bool,
}

impl Default for LightningHitter {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            collides_air: true,
            collides_ground: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightningConfig {
    pub seed: i32,
    pub damage: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub length: i32,
    pub hitter: Option<LightningHitter>,
}

impl LightningConfig {
    pub const fn new(seed: i32, x: f32, y: f32, rotation: f32, length: i32) -> Self {
        Self {
            seed,
            damage: 0.0,
            x,
            y,
            rotation,
            length,
            hitter: None,
        }
    }

    pub const fn with_damage(mut self, damage: f32) -> Self {
        self.damage = damage;
        self
    }

    pub const fn with_hitter(mut self, hitter: LightningHitter) -> Self {
        self.hitter = Some(hitter);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightningSpawnPlan {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub damage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightningInsulatorHit {
    pub tile_x: i32,
    pub tile_y: i32,
}

impl LightningInsulatorHit {
    pub const fn new(tile_x: i32, tile_y: i32) -> Self {
        Self { tile_x, tile_y }
    }

    pub fn world_point(self) -> LightningPoint {
        LightningPoint::new(
            self.tile_x as f32 * TILE_SIZE as f32,
            self.tile_y as f32 * TILE_SIZE as f32,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightningPlan {
    pub lines: Vec<LightningPoint>,
    pub spawns: Vec<LightningSpawnPlan>,
    pub hit_unit_ids: Vec<i32>,
    pub final_x: f32,
    pub final_y: f32,
    pub final_rotation: f32,
    pub stopped_by_insulator: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LightningSeedState {
    pub last_seed: i32,
}

impl LightningSeedState {
    pub fn next_seed(&mut self) -> i32 {
        let seed = self.last_seed;
        self.last_seed = self.last_seed.wrapping_add(1);
        seed
    }
}

pub fn create_lightning_plan<F>(
    config: LightningConfig,
    targets: &[LightningTarget],
    mut insulated_hit: F,
) -> LightningPlan
where
    F: FnMut(i32, i32, i32, i32) -> Option<LightningInsulatorHit>,
{
    let mut random = LightningRand::new(config.seed);
    let mut hit = HashSet::new();
    let mut hit_unit_ids = Vec::new();
    let mut lines = Vec::new();
    let mut spawns = Vec::new();
    let mut stopped_by_insulator = false;

    let mut x = config.x;
    let mut y = config.y;
    let mut rotation = config.rotation;
    let damage_multiplier = config
        .hitter
        .map(|hitter| hitter.damage_multiplier)
        .unwrap_or(1.0);

    for _ in 0..(config.length / 2).max(0) {
        spawns.push(LightningSpawnPlan {
            x,
            y,
            rotation,
            damage: config.damage * damage_multiplier,
        });
        lines.push(LightningPoint::new(
            x + random.range(JITTER_RANGE),
            y + random.range(JITTER_RANGE),
        ));

        if lines.len() > 1 {
            let from = lines[lines.len() - 2];
            let to = lines[lines.len() - 1];
            if let Some(hit_tile) = insulated_hit(
                World::to_tile(from.x),
                World::to_tile(from.y),
                World::to_tile(to.x),
                World::to_tile(to.y),
            ) {
                *lines.last_mut().expect("line was just pushed") = hit_tile.world_point();
                stopped_by_insulator = true;
                break;
            }
        }

        let target = if hit.len() < MAX_CHAIN {
            find_furthest_target(x, y, targets, &hit, config.hitter)
        } else {
            None
        };

        if let Some(target) = target {
            hit.insert(target.id);
            hit_unit_ids.push(target.id);
            x = target.x;
            y = target.y;
        } else {
            rotation += random.range(ANGLE_JITTER);
            x += trnsx(rotation, HIT_RANGE / 2.0);
            y += trnsy(rotation, HIT_RANGE / 2.0);
        }
    }

    LightningPlan {
        lines,
        spawns,
        hit_unit_ids,
        final_x: x,
        final_y: y,
        final_rotation: rotation,
        stopped_by_insulator,
    }
}

pub fn find_furthest_target<'a>(
    x: f32,
    y: f32,
    targets: &'a [LightningTarget],
    already_hit: &HashSet<i32>,
    hitter: Option<LightningHitter>,
) -> Option<&'a LightningTarget> {
    targets
        .iter()
        .filter(|target| {
            target.enemy
                && !already_hit.contains(&target.id)
                && within_lightning_rect(x, y, target.x, target.y)
                && hitter.map_or(true, |hitter| {
                    target.check_target(hitter.collides_air, hitter.collides_ground)
                })
        })
        .max_by(|a, b| {
            dst2(x, y, a.x, a.y)
                .total_cmp(&dst2(x, y, b.x, b.y))
                .then_with(|| a.id.cmp(&b.id))
        })
}

pub fn within_lightning_rect(center_x: f32, center_y: f32, x: f32, y: f32) -> bool {
    let half = HIT_RANGE / 2.0;
    x >= center_x - half && x <= center_x + half && y >= center_y - half && y <= center_y + half
}

fn dst2(x: f32, y: f32, tx: f32, ty: f32) -> f32 {
    let dx = tx - x;
    let dy = ty - y;
    dx * dx + dy * dy
}

fn trnsx(angle_degrees: f32, length: f32) -> f32 {
    angle_degrees.to_radians().cos() * length
}

fn trnsy(angle_degrees: f32, length: f32) -> f32 {
    angle_degrees.to_radians().sin() * length
}

#[derive(Debug, Clone)]
struct LightningRand {
    state: u64,
}

impl LightningRand {
    fn new(seed: i32) -> Self {
        Self {
            state: seed as u64 ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn range(&mut self, range: f32) -> f32 {
        (self.next_f32() * 2.0 - 1.0) * range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_insulator(_: i32, _: i32, _: i32, _: i32) -> Option<LightningInsulatorHit> {
        None
    }

    #[test]
    fn lightning_plan_is_deterministic_for_same_seed() {
        let targets = [LightningTarget::new(1, 10.0, 0.0)];
        let config = LightningConfig::new(7, 0.0, 0.0, 0.0, 8).with_damage(12.0);

        let first = create_lightning_plan(config, &targets, no_insulator);
        let second = create_lightning_plan(config, &targets, no_insulator);

        assert_eq!(first, second);
        assert_eq!(first.spawns[0].damage, 12.0);
    }

    #[test]
    fn lightning_selects_furthest_target_and_never_exceeds_max_chain() {
        let targets = (0..12)
            .map(|index| LightningTarget::new(index, index as f32 + 1.0, 0.0))
            .collect::<Vec<_>>();

        let plan = create_lightning_plan(
            LightningConfig::new(1, 0.0, 0.0, 0.0, 40),
            &targets,
            no_insulator,
        );

        assert_eq!(plan.hit_unit_ids.first(), Some(&11));
        assert_eq!(plan.hit_unit_ids.len(), MAX_CHAIN);
        let unique = plan.hit_unit_ids.iter().copied().collect::<HashSet<_>>();
        assert_eq!(unique.len(), plan.hit_unit_ids.len());
    }

    #[test]
    fn lightning_filters_allies_and_hitter_target_flags() {
        let targets = [
            LightningTarget::new(1, 14.0, 0.0).allied(),
            LightningTarget::new(2, 13.0, 0.0).target_flags(false, false),
            LightningTarget::new(3, 12.0, 0.0).target_flags(false, true),
        ];
        let config = LightningConfig::new(2, 0.0, 0.0, 0.0, 2).with_hitter(LightningHitter {
            damage_multiplier: 2.0,
            collides_air: false,
            collides_ground: true,
        });

        let plan = create_lightning_plan(config, &targets, no_insulator);

        assert_eq!(plan.hit_unit_ids, vec![3]);
        assert_eq!(plan.spawns[0].damage, 0.0);
    }

    #[test]
    fn lightning_skips_already_hit_units() {
        let targets = [LightningTarget::new(1, 5.0, 0.0)];

        let plan = create_lightning_plan(
            LightningConfig::new(3, 0.0, 0.0, 0.0, 10),
            &targets,
            no_insulator,
        );

        assert_eq!(plan.hit_unit_ids, vec![1]);
    }

    #[test]
    fn lightning_stops_and_snaps_last_line_on_insulated_hit() {
        let plan = create_lightning_plan(
            LightningConfig::new(4, 0.0, 0.0, 0.0, 6),
            &[],
            |_, _, _, _| Some(LightningInsulatorHit::new(2, 3)),
        );

        assert!(plan.stopped_by_insulator);
        assert_eq!(plan.lines.last(), Some(&LightningPoint::new(16.0, 24.0)));
        assert_eq!(plan.lines.len(), 2);
    }

    #[test]
    fn lightning_random_walk_advances_when_no_targets_are_available() {
        let plan =
            create_lightning_plan(LightningConfig::new(5, 0.0, 0.0, 0.0, 2), &[], no_insulator);

        assert!(plan.hit_unit_ids.is_empty());
        assert_eq!(plan.lines.len(), 1);
        assert!(plan.final_x > 14.0);
        assert!(plan.final_y.abs() <= 6.0);
    }

    #[test]
    fn seed_state_matches_java_post_increment_shape() {
        let mut state = LightningSeedState::default();
        assert_eq!(state.next_seed(), 0);
        assert_eq!(state.next_seed(), 1);
        state.last_seed = i32::MAX;
        assert_eq!(state.next_seed(), i32::MAX);
        assert_eq!(state.next_seed(), i32::MIN);
    }
}
