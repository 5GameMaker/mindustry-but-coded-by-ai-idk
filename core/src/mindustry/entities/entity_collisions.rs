use crate::mindustry::entities::comp::{HitboxComp, HitboxRect};
use crate::mindustry::vars::TILE_SIZE;

pub const SEGMENT: f32 = 1.0;
pub const MAX_DELTA: f32 = 1000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionPoint {
    pub x: f32,
    pub y: f32,
}

impl CollisionPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionMoveResult {
    pub rect: HitboxRect,
    pub moved_x: f32,
    pub moved_y: f32,
}

pub fn move_check_hitbox<F>(
    entity: &mut HitboxComp,
    deltax: f32,
    deltay: f32,
    solid: F,
) -> CollisionMoveResult
where
    F: Fn(i32, i32) -> bool,
{
    let tile_x = (entity.x / TILE_SIZE as f32).round() as i32;
    let tile_y = (entity.y / TILE_SIZE as f32).round() as i32;
    if solid(tile_x, tile_y) {
        let mut rect = HitboxRect::default();
        entity.hitbox_tile(&mut rect);
        CollisionMoveResult {
            rect,
            moved_x: 0.0,
            moved_y: 0.0,
        }
    } else {
        move_hitbox(entity, deltax, deltay, solid)
    }
}

pub fn move_hitbox<F>(
    entity: &mut HitboxComp,
    deltax: f32,
    deltay: f32,
    solid: F,
) -> CollisionMoveResult
where
    F: Fn(i32, i32) -> bool,
{
    let mut rect = HitboxRect::default();
    entity.hitbox_tile(&mut rect);
    let result = move_rect(rect, deltax, deltay, solid);
    entity.x += result.moved_x;
    entity.y += result.moved_y;
    result
}

pub fn move_rect<F>(
    mut rect: HitboxRect,
    mut deltax: f32,
    mut deltay: f32,
    solid: F,
) -> CollisionMoveResult
where
    F: Fn(i32, i32) -> bool,
{
    if (deltax.abs() < 0.0001 && deltay.abs() < 0.0001) || deltax.is_nan() || deltay.is_nan() {
        return CollisionMoveResult {
            rect,
            moved_x: 0.0,
            moved_y: 0.0,
        };
    }

    deltax = deltax.clamp(-MAX_DELTA, MAX_DELTA);
    deltay = deltay.clamp(-MAX_DELTA, MAX_DELTA);
    let start = rect;
    let radius = ((rect.width / TILE_SIZE as f32).round() as i32).max(1);

    let mut moved_x = false;
    while deltax.abs() > 0.0 || !moved_x {
        moved_x = true;
        let step = deltax.abs().min(SEGMENT) * deltax.signum();
        rect = move_delta_rect(rect, step, 0.0, radius, true, &solid);

        if deltax.abs() >= SEGMENT {
            deltax -= SEGMENT * deltax.signum();
        } else {
            deltax = 0.0;
        }
    }

    let mut moved_y = false;
    while deltay.abs() > 0.0 || !moved_y {
        moved_y = true;
        let step = deltay.abs().min(SEGMENT) * deltay.signum();
        rect = move_delta_rect(rect, 0.0, step, radius, false, &solid);

        if deltay.abs() >= SEGMENT {
            deltay -= SEGMENT * deltay.signum();
        } else {
            deltay = 0.0;
        }
    }

    CollisionMoveResult {
        rect,
        moved_x: rect.x - start.x,
        moved_y: rect.y - start.y,
    }
}

pub fn move_delta_rect<F>(
    rect: HitboxRect,
    deltax: f32,
    deltay: f32,
    radius: i32,
    resolve_x: bool,
    solid: F,
) -> HitboxRect
where
    F: Fn(i32, i32) -> bool,
{
    let mut moved = rect;
    moved.x += deltax;
    moved.y += deltay;

    let tile_x = ((moved.x + moved.width / 2.0) / TILE_SIZE as f32).round() as i32;
    let tile_y = ((moved.y + moved.height / 2.0) / TILE_SIZE as f32).round() as i32;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let wx = dx + tile_x;
            let wy = dy + tile_y;
            if solid(wx, wy) {
                let tile = tile_rect(wx, wy);
                if rect_overlaps(moved, tile) {
                    let overlap = overlap_resolution(moved, tile, resolve_x);
                    moved.x += overlap.x;
                    moved.y += overlap.y;
                }
            }
        }
    }

    moved
}

pub fn overlaps_tile<F>(rect: HitboxRect, solid: F) -> bool
where
    F: Fn(i32, i32) -> bool,
{
    let center_x = rect.x + rect.width / 2.0;
    let center_y = rect.y + rect.height / 2.0;
    let radius = ((rect.width / TILE_SIZE as f32).round() as i32).max(1);
    let tile_x = (center_x / TILE_SIZE as f32).round() as i32;
    let tile_y = (center_y / TILE_SIZE as f32).round() as i32;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let wx = dx + tile_x;
            let wy = dy + tile_y;
            if solid(wx, wy) && rect_overlaps(tile_rect(wx, wy), rect) {
                return true;
            }
        }
    }

    false
}

pub fn collide(
    rect1: HitboxRect,
    velocity1: CollisionPoint,
    rect2: HitboxRect,
    velocity2: CollisionPoint,
) -> Option<CollisionPoint> {
    let px = velocity1.x;
    let py = velocity1.y;
    let vx = velocity1.x - velocity2.x;
    let vy = velocity1.y - velocity2.y;

    let (x_inv_entry, x_inv_exit) = if vx > 0.0 {
        (
            rect2.x - (rect1.x + rect1.width),
            (rect2.x + rect2.width) - rect1.x,
        )
    } else {
        (
            (rect2.x + rect2.width) - rect1.x,
            rect2.x - (rect1.x + rect1.width),
        )
    };

    let (y_inv_entry, y_inv_exit) = if vy > 0.0 {
        (
            rect2.y - (rect1.y + rect1.height),
            (rect2.y + rect2.height) - rect1.y,
        )
    } else {
        (
            (rect2.y + rect2.height) - rect1.y,
            rect2.y - (rect1.y + rect1.height),
        )
    };

    let x_entry = x_inv_entry / vx;
    let x_exit = x_inv_exit / vx;
    let y_entry = y_inv_entry / vy;
    let y_exit = y_inv_exit / vy;

    let entry_time = x_entry.max(y_entry);
    let exit_time = x_exit.min(y_exit);

    if entry_time > exit_time || x_exit < 0.0 || y_exit < 0.0 || x_entry > 1.0 || y_entry > 1.0 {
        None
    } else {
        Some(CollisionPoint::new(
            rect1.x + rect1.width / 2.0 + px * entry_time,
            rect1.y + rect1.height / 2.0 + py * entry_time,
        ))
    }
}

pub fn solid(tile_exists: bool, tile_solid: bool) -> bool {
    !tile_exists || tile_solid
}

pub fn water_solid(tile_exists: bool, tile_solid: bool, floor_is_liquid: bool) -> bool {
    !tile_exists || tile_solid || !floor_is_liquid
}

pub fn legs_solid(tile_exists: bool, tile_leg_solid: bool) -> bool {
    !tile_exists || tile_leg_solid
}

fn tile_rect(tile_x: i32, tile_y: i32) -> HitboxRect {
    let mut rect = HitboxRect::default();
    rect.set_centered(
        tile_x as f32 * TILE_SIZE as f32,
        tile_y as f32 * TILE_SIZE as f32,
        TILE_SIZE as f32,
        TILE_SIZE as f32,
    );
    rect
}

fn rect_overlaps(a: HitboxRect, b: HitboxRect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn overlap_resolution(a: HitboxRect, b: HitboxRect, resolve_x: bool) -> CollisionPoint {
    if resolve_x {
        let acx = a.x + a.width / 2.0;
        let bcx = b.x + b.width / 2.0;
        if acx < bcx {
            CollisionPoint::new(b.x - (a.x + a.width), 0.0)
        } else {
            CollisionPoint::new((b.x + b.width) - a.x, 0.0)
        }
    } else {
        let acy = a.y + a.height / 2.0;
        let bcy = b.y + b.height / 2.0;
        if acy < bcy {
            CollisionPoint::new(0.0, b.y - (a.y + a.height))
        } else {
            CollisionPoint::new(0.0, (b.y + b.height) - a.y)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> HitboxRect {
        HitboxRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn swept_collision_matches_java_entry_time_point() {
        let hit = collide(
            rect(0.0, 0.0, 2.0, 2.0),
            CollisionPoint::new(10.0, 10.0),
            rect(5.0, 5.0, 2.0, 2.0),
            CollisionPoint::new(0.0, 0.0),
        )
        .unwrap();

        assert_eq!(hit, CollisionPoint::new(4.0, 4.0));
        assert!(collide(
            rect(0.0, 0.0, 2.0, 2.0),
            CollisionPoint::new(-10.0, -10.0),
            rect(5.0, 5.0, 2.0, 2.0),
            CollisionPoint::new(0.0, 0.0),
        )
        .is_none());
    }

    #[test]
    fn move_rect_clamps_delta_and_stops_against_solid_tiles() {
        let start = rect(-2.0, -2.0, 4.0, 4.0);

        let free = move_rect(start, 5.0, 0.0, |_, _| false);
        assert_eq!(free.moved_x, 5.0);
        assert_eq!(free.rect.x, 3.0);

        let blocked = move_rect(start, 10.0, 0.0, |x, y| x == 1 && y == 0);
        assert_eq!(blocked.moved_x, 2.0);
        assert_eq!(blocked.rect.x, 0.0);

        let clamped = move_rect(start, 5000.0, 0.0, |_, _| false);
        assert_eq!(clamped.moved_x, MAX_DELTA);
    }

    #[test]
    fn move_hitbox_translates_entity_by_resolved_tile_rect_delta() {
        let mut entity = HitboxComp::new(0.0, 0.0, 6.0);

        let result = move_hitbox(&mut entity, 4.0, 3.0, |_, _| false);

        assert_eq!((result.moved_x, result.moved_y), (4.0, 3.0));
        assert_eq!((entity.x, entity.y), (4.0, 3.0));
    }

    #[test]
    fn overlaps_tile_checks_nearby_solid_tile_rectangles() {
        let mut test = HitboxRect::default();
        test.set_centered(8.0, 0.0, 4.0, 4.0);

        assert!(overlaps_tile(test, |x, y| x == 1 && y == 0));
        assert!(!overlaps_tile(test, |x, y| x == 2 && y == 0));
    }

    #[test]
    fn solid_predicates_match_entity_collision_helpers() {
        assert!(solid(false, false));
        assert!(solid(true, true));
        assert!(!solid(true, false));

        assert!(water_solid(false, false, true));
        assert!(water_solid(true, true, true));
        assert!(water_solid(true, false, false));
        assert!(!water_solid(true, false, true));

        assert!(legs_solid(false, false));
        assert!(legs_solid(true, true));
        assert!(!legs_solid(true, false));
    }
}
