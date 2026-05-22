#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartParams {
    pub warmup: f32,
    pub reload: f32,
    pub smooth_reload: f32,
    pub heat: f32,
    pub recoil: f32,
    pub life: f32,
    pub charge: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub side_override: i32,
    pub side_multiplier: i32,
}

impl Default for PartParams {
    fn default() -> Self {
        Self {
            warmup: 0.0,
            reload: 0.0,
            smooth_reload: 0.0,
            heat: 0.0,
            recoil: 0.0,
            life: 0.0,
            charge: 0.0,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            side_override: -1,
            side_multiplier: 1,
        }
    }
}

impl PartParams {
    pub fn set(
        &mut self,
        warmup: f32,
        reload: f32,
        smooth_reload: f32,
        heat: f32,
        recoil: f32,
        charge: f32,
        x: f32,
        y: f32,
        rotation: f32,
    ) -> &mut Self {
        self.warmup = warmup;
        self.reload = reload;
        self.smooth_reload = smooth_reload;
        self.heat = heat;
        self.recoil = recoil;
        self.charge = charge;
        self.x = x;
        self.y = y;
        self.rotation = rotation;
        self.side_override = -1;
        self.life = 0.0;
        self.side_multiplier = 1;
        self
    }

    pub fn set_recoil(&mut self, recoil: f32) -> &mut Self {
        self.recoil = recoil;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PartProgress {
    Reload,
    SmoothReload,
    Warmup,
    Charge,
    Recoil,
    Heat,
    Life,
    Time,
    Constant(f32),
    Inv(Box<PartProgress>),
    Slope(Box<PartProgress>),
    Clamp(Box<PartProgress>),
    Add(Box<PartProgress>, Box<PartProgress>),
    AddValue(Box<PartProgress>, f32),
    Delay(Box<PartProgress>, f32),
    Curve(Box<PartProgress>, f32, f32),
    Sustain(Box<PartProgress>, f32, f32, f32),
    Shorten(Box<PartProgress>, f32),
    Compress(Box<PartProgress>, f32, f32),
    Blend(Box<PartProgress>, Box<PartProgress>, f32),
    Mul(Box<PartProgress>, Box<PartProgress>),
    MulValue(Box<PartProgress>, f32),
    Min(Box<PartProgress>, Box<PartProgress>),
    Sin(Box<PartProgress>, f32, f32, f32),
    AbsSin(Box<PartProgress>, f32, f32),
    Mod(Box<PartProgress>, f32),
    Loop(Box<PartProgress>, f32),
}

impl PartProgress {
    pub fn constant(value: f32) -> Self {
        Self::Constant(value)
    }

    pub fn get(&self, params: &PartParams, time: f32) -> f32 {
        match self {
            Self::Reload => params.reload,
            Self::SmoothReload => params.smooth_reload,
            Self::Warmup => params.warmup,
            Self::Charge => params.charge,
            Self::Recoil => params.recoil,
            Self::Heat => params.heat,
            Self::Life => params.life,
            Self::Time => time,
            Self::Constant(value) => *value,
            Self::Inv(progress) => 1.0 - progress.get(params, time),
            Self::Slope(progress) => slope(progress.get(params, time)),
            Self::Clamp(progress) => clamp01(progress.get(params, time)),
            Self::Add(left, right) => left.get(params, time) + right.get(params, time),
            Self::AddValue(progress, amount) => progress.get(params, time) + amount,
            Self::Delay(progress, amount) => (progress.get(params, time) - amount) / (1.0 - amount),
            Self::Curve(progress, offset, duration) => {
                (progress.get(params, time) - offset) / duration
            }
            Self::Sustain(progress, offset, grow, sustain) => {
                let val = progress.get(params, time) - offset;
                (val.max(0.0) / grow).min((grow + sustain + grow - val) / grow)
            }
            Self::Shorten(progress, amount) => progress.get(params, time) / (1.0 - amount),
            Self::Compress(progress, start, end) => curve(progress.get(params, time), *start, *end),
            Self::Blend(left, right, amount) => {
                lerp(left.get(params, time), right.get(params, time), *amount)
            }
            Self::Mul(left, right) => left.get(params, time) * right.get(params, time),
            Self::MulValue(progress, amount) => progress.get(params, time) * amount,
            Self::Min(left, right) => left.get(params, time).min(right.get(params, time)),
            Self::Sin(progress, offset, scl, mag) => {
                progress.get(params, time) + ((time + offset) / scl).sin() * mag
            }
            Self::AbsSin(progress, scl, mag) => {
                progress.get(params, time) + (time / scl).sin().abs() * mag
            }
            Self::Mod(progress, amount) => progress.get(params, time).rem_euclid(*amount),
            Self::Loop(progress, loop_time) => {
                (progress.get(params, time) / loop_time).rem_euclid(1.0)
            }
        }
    }

    pub fn get_clamp(&self, params: &PartParams, time: f32, clamp: bool) -> f32 {
        let value = self.get(params, time);
        if clamp {
            clamp01(value)
        } else {
            value
        }
    }

    pub fn inv(self) -> Self {
        Self::Inv(Box::new(self))
    }

    pub fn slope(self) -> Self {
        Self::Slope(Box::new(self))
    }

    pub fn clamp(self) -> Self {
        Self::Clamp(Box::new(self))
    }

    pub fn add_value(self, amount: f32) -> Self {
        Self::AddValue(Box::new(self), amount)
    }

    pub fn add_progress(self, other: Self) -> Self {
        Self::Add(Box::new(self), Box::new(other))
    }

    pub fn delay(self, amount: f32) -> Self {
        Self::Delay(Box::new(self), amount)
    }

    pub fn curve(self, offset: f32, duration: f32) -> Self {
        Self::Curve(Box::new(self), offset, duration)
    }

    pub fn sustain(self, offset: f32, grow: f32, sustain: f32) -> Self {
        Self::Sustain(Box::new(self), offset, grow, sustain)
    }

    pub fn shorten(self, amount: f32) -> Self {
        Self::Shorten(Box::new(self), amount)
    }

    pub fn compress(self, start: f32, end: f32) -> Self {
        Self::Compress(Box::new(self), start, end)
    }

    pub fn blend(self, other: Self, amount: f32) -> Self {
        Self::Blend(Box::new(self), Box::new(other), amount)
    }

    pub fn mul_progress(self, other: Self) -> Self {
        Self::Mul(Box::new(self), Box::new(other))
    }

    pub fn mul_value(self, amount: f32) -> Self {
        Self::MulValue(Box::new(self), amount)
    }

    pub fn min(self, other: Self) -> Self {
        Self::Min(Box::new(self), Box::new(other))
    }

    pub fn sin(self, offset: f32, scl: f32, mag: f32) -> Self {
        Self::Sin(Box::new(self), offset, scl, mag)
    }

    pub fn absin(self, scl: f32, mag: f32) -> Self {
        Self::AbsSin(Box::new(self), scl, mag)
    }

    pub fn modulo(self, amount: f32) -> Self {
        Self::Mod(Box::new(self), amount)
    }

    pub fn looped(self, time: f32) -> Self {
        Self::Loop(Box::new(self), time)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartMove {
    pub progress: PartProgress,
    pub x: f32,
    pub y: f32,
    pub gx: f32,
    pub gy: f32,
    pub rot: f32,
}

impl PartMove {
    pub fn new(progress: PartProgress, x: f32, y: f32, gx: f32, gy: f32, rot: f32) -> Self {
        Self {
            progress,
            x,
            y,
            gx,
            gy,
            rot,
        }
    }

    pub fn local(progress: PartProgress, x: f32, y: f32, rot: f32) -> Self {
        Self::new(progress, x, y, 0.0, 0.0, rot)
    }
}

impl Default for PartMove {
    fn default() -> Self {
        Self::local(PartProgress::Warmup, 0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawPartConfig {
    pub turret_shading: bool,
    pub under: bool,
    pub weapon_index: i32,
    pub recoil_index: i32,
}

impl Default for DrawPartConfig {
    fn default() -> Self {
        Self {
            turret_shading: false,
            under: false,
            weapon_index: 0,
            recoil_index: -1,
        }
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn slope(value: f32) -> f32 {
    1.0 - (value * 2.0 - 1.0).abs()
}

fn curve(value: f32, start: f32, end: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return if value >= end { 1.0 } else { 0.0 };
    }
    ((value - start) / (end - start)).clamp(0.0, 1.0)
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

#[cfg(test)]
mod tests {
    use super::{DrawPartConfig, PartMove, PartParams, PartProgress};

    #[test]
    fn part_params_set_and_recoil_match_java_mutators() {
        let mut params = PartParams {
            life: 0.8,
            side_override: 3,
            side_multiplier: -1,
            ..Default::default()
        };
        params.set(0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 7.0, 8.0, 90.0);
        assert_eq!(params.warmup, 0.1);
        assert_eq!(params.reload, 0.2);
        assert_eq!(params.smooth_reload, 0.3);
        assert_eq!(params.heat, 0.4);
        assert_eq!(params.recoil, 0.5);
        assert_eq!(params.charge, 0.6);
        assert_eq!((params.x, params.y, params.rotation), (7.0, 8.0, 90.0));
        assert_eq!(params.life, 0.0);
        assert_eq!(params.side_override, -1);
        assert_eq!(params.side_multiplier, 1);

        params.set_recoil(2.5);
        assert_eq!(params.recoil, 2.5);
    }

    #[test]
    fn part_progress_variants_and_combinators_are_pure() {
        let mut params = PartParams::default();
        params.set(0.25, 0.75, 0.5, 0.3, 0.2, 0.9, 0.0, 0.0, 0.0);
        params.life = 0.4;

        assert_eq!(PartProgress::Warmup.get(&params, 10.0), 0.25);
        assert_eq!(PartProgress::Reload.inv().get(&params, 10.0), 0.25);
        assert_eq!(PartProgress::Warmup.slope().get(&params, 10.0), 0.5);
        assert_eq!(PartProgress::constant(2.0).clamp().get(&params, 10.0), 1.0);
        assert_eq!(PartProgress::Warmup.add_value(0.5).get(&params, 10.0), 0.75);
        assert_eq!(
            PartProgress::Warmup
                .add_progress(PartProgress::Heat)
                .get(&params, 10.0),
            0.55
        );
        assert_eq!(
            PartProgress::Reload.delay(0.25).get(&params, 10.0),
            2.0 / 3.0
        );
        assert_eq!(
            PartProgress::Reload.compress(0.5, 1.0).get(&params, 10.0),
            0.5
        );
        assert_eq!(
            PartProgress::Warmup
                .blend(PartProgress::Reload, 0.5)
                .get(&params, 10.0),
            0.5
        );
        assert_eq!(
            PartProgress::Reload
                .mul_progress(PartProgress::Warmup)
                .get(&params, 10.0),
            0.1875
        );
        assert_eq!(PartProgress::Reload.mul_value(2.0).get(&params, 10.0), 1.5);
        assert_eq!(
            PartProgress::Reload
                .min(PartProgress::Warmup)
                .get(&params, 10.0),
            0.25
        );
        assert_eq!(
            PartProgress::constant(5.5).modulo(2.0).get(&params, 10.0),
            1.5
        );
        assert_eq!(PartProgress::Time.looped(4.0).get(&params, 10.0), 0.5);
        assert_eq!(PartProgress::Life.get_clamp(&params, 10.0, true), 0.4);
    }

    #[test]
    fn part_move_and_draw_part_config_defaults_match_java_fields() {
        let movement = PartMove::local(PartProgress::Recoil, 1.0, 2.0, 3.0);
        assert_eq!(movement.progress, PartProgress::Recoil);
        assert_eq!(
            (
                movement.x,
                movement.y,
                movement.gx,
                movement.gy,
                movement.rot
            ),
            (1.0, 2.0, 0.0, 0.0, 3.0)
        );

        let config = DrawPartConfig::default();
        assert!(!config.turret_shading);
        assert!(!config.under);
        assert_eq!(config.weapon_index, 0);
        assert_eq!(config.recoil_index, -1);
    }
}
