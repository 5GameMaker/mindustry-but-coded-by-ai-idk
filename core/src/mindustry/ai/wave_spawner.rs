use crate::mindustry::vars::TILE_SIZE;

pub const SPAWN_MARGIN: f32 = 0.0;
pub const CORE_MARGIN: f32 = TILE_SIZE as f32 * 2.0;
pub const MAX_CORE_SPAWN_STEPS: i32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnTile {
    pub pos: i32,
    pub x: i32,
    pub y: i32,
}

impl SpawnTile {
    pub const fn new(pos: i32, x: i32, y: i32) -> Self {
        Self { pos, x, y }
    }

    pub fn world_x(self) -> f32 {
        self.x as f32 * TILE_SIZE as f32
    }

    pub fn world_y(self) -> f32 {
        self.y as f32 * TILE_SIZE as f32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroundSpawn {
    pub x: f32,
    pub y: f32,
    pub shockwave: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlyerSpawn {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnEffectPlan {
    pub statuses: Vec<(&'static str, f32)>,
    pub unload: bool,
    pub event: &'static str,
    pub call_spawn_effect: bool,
}

pub fn player_near(
    has_spawns: bool,
    player_dead: bool,
    player_x: f32,
    player_y: f32,
    player_team: i32,
    wave_team: i32,
    drop_zone_radius: f32,
    spawns: &[SpawnTile],
) -> bool {
    has_spawns
        && !player_dead
        && player_team != wave_team
        && spawns.iter().any(|spawn| {
            distance(spawn.world_x(), spawn.world_y(), player_x, player_y) < drop_zone_radius
        })
}

pub fn ground_spawns(spawns: &[SpawnTile], filter_pos: Option<i32>) -> Vec<GroundSpawn> {
    spawns
        .iter()
        .filter(|spawn| filter_pos.map_or(true, |filter| filter == spawn.pos))
        .map(|spawn| GroundSpawn {
            x: spawn.world_x(),
            y: spawn.world_y(),
            shockwave: true,
        })
        .collect()
}

pub fn flyer_spawns(
    spawns: &[SpawnTile],
    filter_pos: Option<i32>,
    air_use_spawns: bool,
    world_width: i32,
    world_height: i32,
) -> Vec<FlyerSpawn> {
    spawns
        .iter()
        .filter(|spawn| filter_pos.map_or(true, |filter| filter == spawn.pos))
        .map(|spawn| {
            if air_use_spawns {
                FlyerSpawn {
                    x: spawn.world_x(),
                    y: spawn.world_y(),
                }
            } else {
                flyer_edge_spawn(*spawn, world_width, world_height)
            }
        })
        .collect()
}

pub fn flyer_edge_spawn(spawn: SpawnTile, world_width: i32, world_height: i32) -> FlyerSpawn {
    let center_x = world_width as f32 / 2.0;
    let center_y = world_height as f32 / 2.0;
    let angle = angle_degrees(center_x, center_y, spawn.x as f32, spawn.y as f32);
    let trns = world_width.max(world_height) as f32 * std::f32::consts::SQRT_2 * TILE_SIZE as f32;
    let world_w = world_width as f32 * TILE_SIZE as f32;
    let world_h = world_height as f32 * TILE_SIZE as f32;

    FlyerSpawn {
        x: (world_w / 2.0 + trnsx(angle, trns)).clamp(-SPAWN_MARGIN, world_w + SPAWN_MARGIN),
        y: (world_h / 2.0 + trnsy(angle, trns)).clamp(-SPAWN_MARGIN, world_h + SPAWN_MARGIN),
    }
}

pub fn count_ground_spawns(spawns: &[SpawnTile]) -> usize {
    ground_spawns(spawns, None).len()
}

pub fn count_flyer_spawns(spawns: &[SpawnTile]) -> usize {
    spawns.len()
}

pub fn spawn_rotation(x: f32, y: f32, world_width: i32, world_height: i32) -> f32 {
    angle_degrees(
        x,
        y,
        world_width as f32 / 2.0 * TILE_SIZE as f32,
        world_height as f32 / 2.0 * TILE_SIZE as f32,
    )
}

pub fn is_spawning(spawning: bool, net_client: bool) -> bool {
    spawning && !net_client
}

pub fn spawn_effect_plan() -> SpawnEffectPlan {
    SpawnEffectPlan {
        statuses: vec![("unmoving", 30.0), ("invincible", 60.0)],
        unload: true,
        event: "UnitSpawnEvent",
        call_spawn_effect: true,
    }
}

fn distance(x: f32, y: f32, tx: f32, ty: f32) -> f32 {
    let dx = tx - x;
    let dy = ty - y;
    (dx * dx + dy * dy).sqrt()
}

fn angle_degrees(x: f32, y: f32, tx: f32, ty: f32) -> f32 {
    (ty - y).atan2(tx - x).to_degrees()
}

fn trnsx(angle_degrees: f32, length: f32) -> f32 {
    angle_degrees.to_radians().cos() * length
}

fn trnsy(angle_degrees: f32, length: f32) -> f32 {
    angle_degrees.to_radians().sin() * length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_near_matches_drop_zone_and_team_rules() {
        let spawns = [SpawnTile::new(1, 2, 3)];

        assert!(player_near(true, false, 16.0, 24.0, 1, 2, 10.0, &spawns));
        assert!(!player_near(true, false, 100.0, 100.0, 1, 2, 10.0, &spawns));
        assert!(!player_near(true, false, 16.0, 24.0, 2, 2, 10.0, &spawns));
        assert!(!player_near(true, true, 16.0, 24.0, 1, 2, 10.0, &spawns));
    }

    #[test]
    fn ground_and_flyer_spawn_counts_and_filters_use_spawn_tiles() {
        let spawns = [SpawnTile::new(11, 1, 2), SpawnTile::new(22, 3, 4)];

        assert_eq!(count_ground_spawns(&spawns), 2);
        assert_eq!(count_flyer_spawns(&spawns), 2);
        assert_eq!(
            ground_spawns(&spawns, Some(22)),
            vec![GroundSpawn {
                x: 24.0,
                y: 32.0,
                shockwave: true,
            }]
        );
        assert_eq!(
            flyer_spawns(&spawns, Some(11), true, 10, 10),
            vec![FlyerSpawn { x: 8.0, y: 16.0 }]
        );
    }

    #[test]
    fn flyer_edge_spawn_projects_spawn_to_world_edge_when_air_does_not_use_spawns() {
        let spawn = flyer_edge_spawn(SpawnTile::new(1, 0, 5), 10, 10);

        assert_eq!(spawn.x, 0.0);
        assert!((0.0..=80.0).contains(&spawn.y));
    }

    #[test]
    fn spawn_rotation_points_toward_world_center() {
        assert_eq!(spawn_rotation(0.0, 40.0, 10, 10), 0.0);
        assert_eq!(spawn_rotation(40.0, 0.0, 10, 10), 90.0);
    }

    #[test]
    fn spawning_state_and_spawn_effect_plan_match_java_side_effect_shape() {
        assert!(is_spawning(true, false));
        assert!(!is_spawning(true, true));
        assert!(!is_spawning(false, false));

        let plan = spawn_effect_plan();
        assert_eq!(
            plan.statuses,
            vec![("unmoving", 30.0), ("invincible", 60.0)]
        );
        assert!(plan.unload);
        assert_eq!(plan.event, "UnitSpawnEvent");
        assert!(plan.call_spawn_effect);
    }
}
