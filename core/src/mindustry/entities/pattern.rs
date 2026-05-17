#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Shot {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub delay: f32,
}

impl Shot {
    pub const fn new(x: f32, y: f32, rotation: f32, delay: f32) -> Self {
        Self {
            x,
            y,
            rotation,
            delay,
        }
    }
}

pub trait BulletHandler {
    fn shoot(&mut self, shot: Shot);
}

impl<F> BulletHandler for F
where
    F: FnMut(Shot),
{
    fn shoot(&mut self, shot: Shot) {
        self(shot);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShootPattern {
    pub shots: i32,
    pub first_shot_delay: f32,
    pub shot_delay: f32,
}

impl ShootPattern {
    pub const fn new() -> Self {
        Self {
            shots: 1,
            first_shot_delay: 0.0,
            shot_delay: 0.0,
        }
    }

    pub fn shoot<H>(
        &self,
        _total_shots: i32,
        handler: &mut H,
        mut barrel_incrementer: Option<&mut dyn FnMut()>,
    ) where
        H: BulletHandler + ?Sized,
    {
        for i in 0..self.shots {
            handler.shoot(Shot::new(
                0.0,
                0.0,
                0.0,
                self.first_shot_delay + self.shot_delay * i as f32,
            ));
            if let Some(incrementer) = barrel_incrementer.as_mut() {
                (*incrementer)();
            }
        }
    }

    pub fn flip(&mut self) {}

    pub fn copy(&self) -> Self {
        self.clone()
    }
}

impl Default for ShootPattern {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShootAlternate {
    pub pattern: ShootPattern,
    pub barrels: i32,
    pub spread: f32,
    pub barrel_offset: i32,
    pub mirror: bool,
}

impl ShootAlternate {
    pub const fn new(spread: f32) -> Self {
        Self {
            pattern: ShootPattern::new(),
            barrels: 2,
            spread,
            barrel_offset: 0,
            mirror: false,
        }
    }

    pub fn shoot<H>(
        &self,
        total_shots: i32,
        handler: &mut H,
        mut barrel_incrementer: Option<&mut dyn FnMut()>,
    ) where
        H: BulletHandler + ?Sized,
    {
        let barrels = self.barrels.max(1);
        let mirror_sign = if self.mirror { -1.0 } else { 1.0 };

        for i in 0..self.pattern.shots {
            let index = (total_shots + i + self.barrel_offset).rem_euclid(barrels) as f32
                - (barrels - 1) as f32 / 2.0;
            handler.shoot(Shot::new(
                index * self.spread * mirror_sign,
                0.0,
                0.0,
                self.pattern.first_shot_delay + self.pattern.shot_delay * i as f32,
            ));
            if let Some(incrementer) = barrel_incrementer.as_mut() {
                (*incrementer)();
            }
        }
    }

    pub fn flip(&mut self) {
        self.mirror = !self.mirror;
    }
}

impl Default for ShootAlternate {
    fn default() -> Self {
        Self::new(5.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShootBarrel {
    pub pattern: ShootPattern,
    /// barrels [in x, y, rotation] format.
    pub barrels: Vec<f32>,
    pub barrel_offset: i32,
}

impl ShootBarrel {
    pub fn new(barrels: Vec<f32>) -> Self {
        Self {
            pattern: ShootPattern::new(),
            barrels,
            barrel_offset: 0,
        }
    }

    pub fn shoot<H>(
        &self,
        total_shots: i32,
        handler: &mut H,
        mut barrel_incrementer: Option<&mut dyn FnMut()>,
    ) where
        H: BulletHandler + ?Sized,
    {
        let barrel_count = self.barrels.len() / 3;
        if barrel_count == 0 {
            return;
        }

        for i in 0..self.pattern.shots {
            let index =
                (i + total_shots + self.barrel_offset).rem_euclid(barrel_count as i32) as usize * 3;
            handler.shoot(Shot::new(
                self.barrels[index],
                self.barrels[index + 1],
                self.barrels[index + 2],
                self.pattern.first_shot_delay + self.pattern.shot_delay * i as f32,
            ));
            if let Some(incrementer) = barrel_incrementer.as_mut() {
                (*incrementer)();
            }
        }
    }

    pub fn flip(&mut self) {
        for i in (0..self.barrels.len()).step_by(3) {
            self.barrels[i] *= -1.0;
            if i + 2 < self.barrels.len() {
                self.barrels[i + 2] *= -1.0;
            }
        }
    }
}

impl Default for ShootBarrel {
    fn default() -> Self {
        Self::new(vec![0.0, 0.0, 0.0])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShootSpread {
    pub pattern: ShootPattern,
    /// spread between bullets, in degrees.
    pub spread: f32,
}

impl ShootSpread {
    pub fn new(shots: i32, spread: f32) -> Self {
        let mut pattern = ShootPattern::new();
        pattern.shots = shots;
        Self { pattern, spread }
    }

    pub fn circle(points: i32) -> Self {
        Self::new(points, 360.0 / points as f32)
    }

    pub fn shoot<H>(
        &self,
        _total_shots: i32,
        handler: &mut H,
        _barrel_incrementer: Option<&mut dyn FnMut()>,
    ) where
        H: BulletHandler + ?Sized,
    {
        for i in 0..self.pattern.shots {
            let angle_offset =
                i as f32 * self.spread - (self.pattern.shots - 1) as f32 * self.spread / 2.0;
            handler.shoot(Shot::new(
                0.0,
                0.0,
                angle_offset,
                self.pattern.first_shot_delay + self.pattern.shot_delay * i as f32,
            ));
        }
    }
}

impl Default for ShootSpread {
    fn default() -> Self {
        Self::new(1, 5.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShootSine {
    pub pattern: ShootPattern,
    pub scl: f32,
    pub mag: f32,
}

impl ShootSine {
    pub const fn new(scl: f32, mag: f32) -> Self {
        Self {
            pattern: ShootPattern::new(),
            scl,
            mag,
        }
    }

    pub fn shoot<H>(
        &self,
        total_shots: i32,
        handler: &mut H,
        _barrel_incrementer: Option<&mut dyn FnMut()>,
    ) where
        H: BulletHandler + ?Sized,
    {
        for i in 0..self.pattern.shots {
            let angle_offset = sin_scale(i as f32 + total_shots as f32, self.scl, self.mag);
            handler.shoot(Shot::new(
                0.0,
                0.0,
                angle_offset,
                self.pattern.first_shot_delay + self.pattern.shot_delay * i as f32,
            ));
        }
    }
}

impl Default for ShootSine {
    fn default() -> Self {
        Self::new(4.0, 20.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShotMover {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HelixShot {
    pub shot: Shot,
    pub mover: ShotMover,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShootHelix {
    pub pattern: ShootPattern,
    pub scl: f32,
    pub mag: f32,
    pub offset: f32,
}

impl ShootHelix {
    pub fn new(scl: f32, mag: f32) -> Self {
        Self {
            pattern: ShootPattern::new(),
            scl,
            mag,
            offset: std::f32::consts::PI * 1.25,
        }
    }

    pub fn with_offset(scl: f32, mag: f32, offset: f32) -> Self {
        Self {
            pattern: ShootPattern::new(),
            scl,
            mag,
            offset,
        }
    }

    pub fn shoot_collect(&self, total_shots: i32) -> Vec<HelixShot> {
        let mut out = Vec::new();
        for i in 0..self.pattern.shots {
            for sign in [1.0, -1.0] {
                let time = total_shots as f32 + i as f32;
                out.push(HelixShot {
                    shot: Shot::new(
                        0.0,
                        0.0,
                        0.0,
                        self.pattern.first_shot_delay + self.pattern.shot_delay * i as f32,
                    ),
                    mover: ShotMover {
                        x: 0.0,
                        y: sin_scale(time + self.offset, self.scl, self.mag * sign),
                    },
                });
            }
        }
        out
    }

    pub fn shoot<H>(
        &self,
        total_shots: i32,
        handler: &mut H,
        _barrel_incrementer: Option<&mut dyn FnMut()>,
    ) where
        H: BulletHandler + ?Sized,
    {
        for shot in self.shoot_collect(total_shots) {
            handler.shoot(shot.shot);
        }
    }
}

impl Default for ShootHelix {
    fn default() -> Self {
        Self::new(2.0, 1.5)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShootSummon {
    pub pattern: ShootPattern,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub spread: f32,
}

impl ShootSummon {
    pub const fn new(x: f32, y: f32, radius: f32, spread: f32) -> Self {
        Self {
            pattern: ShootPattern::new(),
            x,
            y,
            radius,
            spread,
        }
    }

    pub fn shoot_with_rng<H, R>(
        &self,
        handler: &mut H,
        mut rng: R,
        _barrel_incrementer: Option<&mut dyn FnMut()>,
    ) where
        H: BulletHandler + ?Sized,
        R: FnMut() -> f32,
    {
        for i in 0..self.pattern.shots {
            let angle = rng() * 360.0;
            let radius = rng() * self.radius;
            let rotation = (rng() * 2.0 - 1.0) * self.spread;
            let radians = angle.to_radians();
            handler.shoot(Shot::new(
                self.x + radians.cos() * radius,
                self.y + radians.sin() * radius,
                rotation,
                self.pattern.first_shot_delay + self.pattern.shot_delay * i as f32,
            ));
        }
    }
}

impl Default for ShootSummon {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

fn sin_scale(value: f32, scl: f32, mag: f32) -> f32 {
    (value / scl).sin() * mag
}

#[cfg(test)]
mod tests {
    use super::{
        ShootAlternate, ShootBarrel, ShootHelix, ShootPattern, ShootSine, ShootSpread, ShootSummon,
        Shot,
    };

    #[test]
    fn shoot_pattern_emits_requested_number_of_shots() {
        let mut pattern = ShootPattern::default();
        pattern.shots = 3;
        pattern.first_shot_delay = 2.0;
        pattern.shot_delay = 1.5;

        let mut shots = Vec::new();
        pattern.shoot(0, &mut |shot| shots.push(shot), None);

        assert_eq!(
            shots,
            vec![
                Shot::new(0.0, 0.0, 0.0, 2.0),
                Shot::new(0.0, 0.0, 0.0, 3.5),
                Shot::new(0.0, 0.0, 0.0, 5.0),
            ]
        );
    }

    #[test]
    fn shoot_alternate_flips_barrel_side_and_runs_incrementer() {
        let mut pattern = ShootAlternate::new(4.0);
        pattern.pattern.shots = 2;
        pattern.barrels = 2;

        let mut shots = Vec::new();
        let mut increments = 0;
        pattern.shoot(
            0,
            &mut |shot| shots.push(shot),
            Some(&mut || {
                increments += 1;
            }),
        );

        assert_eq!(shots[0].x, -2.0);
        assert_eq!(shots[1].x, 2.0);
        assert_eq!(increments, 2);

        pattern.flip();
        shots.clear();
        pattern.shoot(0, &mut |shot| shots.push(shot), None);
        assert_eq!(shots[0].x, 2.0);
        assert_eq!(shots[1].x, -2.0);
    }

    #[test]
    fn shoot_barrel_uses_triplets_and_flips_only_x_and_rotation() {
        let mut pattern = ShootBarrel::new(vec![1.0, 2.0, 3.0, -4.0, -5.0, -6.0]);
        pattern.pattern.shots = 2;
        pattern.barrel_offset = 0;

        let mut shots = Vec::new();
        pattern.shoot(1, &mut |shot| shots.push(shot), None);
        assert_eq!(shots[0], Shot::new(-4.0, -5.0, -6.0, 0.0));
        assert_eq!(shots[1], Shot::new(1.0, 2.0, 3.0, 0.0));

        pattern.flip();
        assert_eq!(pattern.barrels, vec![-1.0, 2.0, -3.0, 4.0, -5.0, 6.0]);
    }

    #[test]
    fn shoot_spread_centers_angle_offsets() {
        let pattern = ShootSpread::new(3, 10.0);
        let mut shots = Vec::new();
        pattern.shoot(0, &mut |shot| shots.push(shot), None);
        assert_eq!(shots[0].rotation, -10.0);
        assert_eq!(shots[1].rotation, 0.0);
        assert_eq!(shots[2].rotation, 10.0);
        assert_eq!(ShootSpread::circle(4).spread, 90.0);
    }

    #[test]
    fn shoot_sine_uses_total_shots_in_angle_formula() {
        let mut pattern = ShootSine::new(2.0, 4.0);
        pattern.pattern.shots = 1;
        let mut shots = Vec::new();
        pattern.shoot(2, &mut |shot| shots.push(shot), None);
        assert!((shots[0].rotation - (1.0f32.sin() * 4.0)).abs() < 0.0001);
    }

    #[test]
    fn shoot_helix_emits_mirrored_movers() {
        let mut pattern = ShootHelix::with_offset(2.0, 3.0, 0.0);
        pattern.pattern.shots = 1;
        let shots = pattern.shoot_collect(2);
        assert_eq!(shots.len(), 2);
        assert!((shots[0].mover.y + shots[1].mover.y).abs() < 0.0001);
    }

    #[test]
    fn shoot_summon_uses_radius_and_spread_from_rng() {
        let mut pattern = ShootSummon::new(1.0, 2.0, 10.0, 30.0);
        pattern.pattern.shots = 1;
        let mut values = [0.25, 0.5, 1.0].into_iter();
        let mut shots = Vec::new();
        pattern.shoot_with_rng(
            &mut |shot| shots.push(shot),
            || values.next().unwrap(),
            None,
        );
        assert!((shots[0].x - 1.0).abs() < 0.0001);
        assert!((shots[0].y - 7.0).abs() < 0.0001);
        assert!((shots[0].rotation - 30.0).abs() < 0.0001);
    }
}
