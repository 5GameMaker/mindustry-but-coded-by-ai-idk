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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectSpawnerRectPlan {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectSpawnerSpawnPlan {
    pub effect: String,
    pub color: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectSpawnerDrawPlan {
    pub debug_rects: Vec<EffectSpawnerRectPlan>,
    pub spawns: Vec<EffectSpawnerSpawnPlan>,
    pub effect_interval_state: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectSpawnerPart {
    pub config: DrawPartConfig,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub mirror: bool,
    pub effect_rot: f32,
    pub effect_rand_rot: f32,
    pub effect_interval: f32,
    pub effect_interval_from: f32,
    pub effect_chance: f32,
    pub effect: String,
    pub effect_color: String,
    pub use_progress: bool,
    pub progress: PartProgress,
    pub debug_draw: bool,
    pub effect_interval_state: f32,
}

impl Default for EffectSpawnerPart {
    fn default() -> Self {
        Self {
            config: DrawPartConfig::default(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            mirror: false,
            effect_rot: 0.0,
            effect_rand_rot: 0.0,
            effect_interval: 0.0,
            effect_interval_from: 0.0,
            effect_chance: 0.1,
            effect: "sparkShoot".into(),
            effect_color: "white".into(),
            use_progress: true,
            progress: PartProgress::Warmup,
            debug_draw: false,
            effect_interval_state: 0.0,
        }
    }
}

impl EffectSpawnerPart {
    pub fn draw_plan(
        &mut self,
        params: &PartParams,
        time: f32,
        delta: f32,
        paused: bool,
        chance_values: &[f32],
        random_offsets: &[(f32, f32, f32)],
    ) -> EffectSpawnerDrawPlan {
        let sides = if self.mirror { 2 } else { 1 };
        let mut debug_rects = Vec::new();
        let mut spawns = Vec::new();

        for i in 0..sides {
            let sign = if i == 0 { 1.0 } else { -1.0 };
            let rot = params.rotation + self.rotation * sign;
            let base = rotate_offset(params.rotation - 90.0, self.x * sign, self.y);
            let x = params.x + base.0;
            let y = params.y + base.1;
            if self.debug_draw {
                debug_rects.push(EffectSpawnerRectPlan {
                    x,
                    y,
                    width: self.width,
                    height: self.height,
                    rotation: rot - 90.0,
                });
            }
        }

        if !paused {
            let progress = self.progress.get_clamp(params, time, true);
            let real_interval = if self.effect_interval_from > 0.0 {
                lerp(self.effect_interval_from, self.effect_interval, progress)
            } else {
                self.effect_interval
            };

            for i in 0..sides {
                let should_spawn = if real_interval > 0.0 {
                    self.effect_interval_state += delta;
                    self.effect_interval_state >= real_interval
                } else {
                    let threshold =
                        self.effect_chance * if self.use_progress { progress } else { 1.0 };
                    chance_values.get(i).copied().unwrap_or(1.0) <= threshold
                };

                if should_spawn {
                    let sign = if i == 0 { 1.0 } else { -1.0 };
                    let rot = params.rotation + self.rotation * sign;
                    let base = rotate_offset(params.rotation - 90.0, self.x * sign, self.y);
                    let random = random_offsets.get(i).copied().unwrap_or((0.0, 0.0, 0.0));
                    let jitter = rotate_offset(rot, random.0, random.1);
                    spawns.push(EffectSpawnerSpawnPlan {
                        effect: self.effect.clone(),
                        color: self.effect_color.clone(),
                        x: params.x + base.0 + jitter.0,
                        y: params.y + base.1 + jitter.1,
                        rotation: rot
                            + self.effect_rot * sign
                            + random.2.clamp(-self.effect_rand_rot, self.effect_rand_rot),
                    });
                    if real_interval > 0.0 {
                        self.effect_interval_state %= real_interval;
                    }
                }
            }
        }

        EffectSpawnerDrawPlan {
            debug_rects,
            spawns,
            effect_interval_state: self.effect_interval_state,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapePartKind {
    Circle,
    Polygon { sides: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapePartDrawItem {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub rotation: f32,
    pub kind: ShapePartKind,
    pub hollow: bool,
    pub stroke: f32,
    pub color: String,
    pub color_to: Option<String>,
    pub color_mix: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapePartDrawPlan {
    pub layer: Option<f32>,
    pub layer_offset: f32,
    pub under_turret_shading: bool,
    pub shapes: Vec<ShapePartDrawItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapePart {
    pub config: DrawPartConfig,
    pub circle: bool,
    pub hollow: bool,
    pub sides: i32,
    pub radius: f32,
    pub radius_to: f32,
    pub stroke: f32,
    pub stroke_to: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub move_x: f32,
    pub move_y: f32,
    pub move_rot: f32,
    pub rotate_speed: f32,
    pub color: String,
    pub color_to: Option<String>,
    pub mirror: bool,
    pub clamp_progress: bool,
    pub progress: PartProgress,
    pub layer: f32,
    pub layer_offset: f32,
}

impl Default for ShapePart {
    fn default() -> Self {
        Self {
            config: DrawPartConfig::default(),
            circle: false,
            hollow: false,
            sides: 3,
            radius: 3.0,
            radius_to: -1.0,
            stroke: 1.0,
            stroke_to: -1.0,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            move_x: 0.0,
            move_y: 0.0,
            move_rot: 0.0,
            rotate_speed: 0.0,
            color: "white".into(),
            color_to: None,
            mirror: false,
            clamp_progress: true,
            progress: PartProgress::Warmup,
            layer: -1.0,
            layer_offset: 0.0,
        }
    }
}

impl ShapePart {
    pub fn draw_plan(&self, params: &PartParams, time: f32) -> ShapePartDrawPlan {
        let prog = self.progress.get_clamp(params, time, self.clamp_progress);
        let base_rot = time * self.rotate_speed;
        let radius = if self.radius_to < 0.0 {
            self.radius
        } else {
            lerp(self.radius, self.radius_to, prog)
        };
        let stroke = if self.stroke_to < 0.0 {
            self.stroke
        } else {
            lerp(self.stroke, self.stroke_to, prog)
        };
        let len = if self.mirror && params.side_override == -1 {
            2
        } else {
            1
        };

        let mut shapes = Vec::with_capacity(len);
        for side in 0..len {
            let i = if params.side_override == -1 {
                side as i32
            } else {
                params.side_override
            };
            let sign = (if i == 0 { 1.0 } else { -1.0 }) * params.side_multiplier as f32;
            let offset = rotate_offset(
                params.rotation - 90.0,
                (self.x + self.move_x * prog) * sign,
                self.y + self.move_y * prog,
            );
            shapes.push(ShapePartDrawItem {
                x: params.x + offset.0,
                y: params.y + offset.1,
                radius,
                rotation: self.move_rot * prog * sign + params.rotation - 90.0 * sign
                    + self.rotation * sign
                    + base_rot * sign,
                kind: if self.circle {
                    ShapePartKind::Circle
                } else {
                    ShapePartKind::Polygon { sides: self.sides }
                },
                hollow: self.hollow,
                stroke,
                color: self.color.clone(),
                color_to: self.color_to.clone(),
                color_mix: prog,
            });
        }

        ShapePartDrawPlan {
            layer: (self.layer > 0.0).then_some(self.layer),
            layer_offset: self.layer_offset,
            under_turret_shading: self.config.under && self.config.turret_shading,
            shapes,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlareTrianglePlan {
    pub color: String,
    pub width: f32,
    pub length: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlarePartDrawPlan {
    pub x: f32,
    pub y: f32,
    pub layer: f32,
    pub triangles: Vec<FlareTrianglePlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlarePart {
    pub config: DrawPartConfig,
    pub sides: i32,
    pub radius: f32,
    pub radius_to: f32,
    pub stroke: f32,
    pub inner_scl: f32,
    pub inner_rad_scl: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub rot_move: f32,
    pub spin_speed: f32,
    pub follow_rotation: bool,
    pub color1: String,
    pub color2: String,
    pub clamp_progress: bool,
    pub progress: PartProgress,
    pub layer: f32,
}

impl Default for FlarePart {
    fn default() -> Self {
        Self {
            config: DrawPartConfig::default(),
            sides: 4,
            radius: 100.0,
            radius_to: -1.0,
            stroke: 6.0,
            inner_scl: 0.5,
            inner_rad_scl: 0.33,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            rot_move: 0.0,
            spin_speed: 0.0,
            follow_rotation: false,
            color1: "techBlue".into(),
            color2: "white".into(),
            clamp_progress: true,
            progress: PartProgress::Warmup,
            layer: 110.0,
        }
    }
}

impl FlarePart {
    pub fn draw_plan(&self, params: &PartParams, time: f32) -> FlarePartDrawPlan {
        let prog = self.progress.get_clamp(params, time, self.clamp_progress);
        let side = if params.side_override == -1 {
            0
        } else {
            params.side_override
        };
        let sign = (if side == 0 { 1.0 } else { -1.0 }) * params.side_multiplier as f32;
        let offset = rotate_offset(params.rotation - 90.0, self.x * sign, self.y);
        let x = params.x + offset.0;
        let y = params.y + offset.1;
        let base_rotation = if self.follow_rotation {
            params.rotation
        } else {
            0.0
        } + self.rot_move * prog
            + self.rotation
            + time * self.spin_speed;
        let radius = if self.radius_to < 0.0 {
            self.radius
        } else {
            lerp(self.radius, self.radius_to, prog)
        };

        let mut triangles = Vec::with_capacity(self.sides.max(0) as usize * 2);
        for color_pass in 0..2 {
            for j in 0..self.sides.max(0) {
                let inner = color_pass == 1;
                triangles.push(FlareTrianglePlan {
                    color: if inner {
                        self.color2.clone()
                    } else {
                        self.color1.clone()
                    },
                    width: if inner {
                        self.stroke * self.inner_scl
                    } else {
                        self.stroke
                    },
                    length: if inner {
                        radius * self.inner_rad_scl
                    } else {
                        radius
                    },
                    rotation: j as f32 * 360.0 / self.sides.max(1) as f32 + base_rotation,
                });
            }
        }

        FlarePartDrawPlan {
            x,
            y,
            layer: self.layer,
            triangles,
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

fn rotate_offset(angle: f32, x: f32, y: f32) -> (f32, f32) {
    let rad = angle.to_radians();
    (x * rad.cos() - y * rad.sin(), x * rad.sin() + y * rad.cos())
}

#[cfg(test)]
mod tests {
    use super::{
        DrawPartConfig, EffectSpawnerPart, EffectSpawnerRectPlan, FlarePart, PartMove, PartParams,
        PartProgress, ShapePart, ShapePartKind,
    };

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

    #[test]
    fn effect_spawner_part_builds_debug_rects_and_effect_spawn_plan() {
        let mut params = PartParams::default();
        params.set(0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 100.0, 200.0, 90.0);

        let mut part = EffectSpawnerPart {
            x: 10.0,
            y: 0.0,
            width: 4.0,
            height: 8.0,
            mirror: true,
            effect_chance: 1.0,
            effect_rot: 0.0,
            effect_rand_rot: 10.0,
            debug_draw: true,
            ..Default::default()
        };

        let plan = part.draw_plan(
            &params,
            0.0,
            1.0,
            false,
            &[0.4, 0.6],
            &[(0.0, 0.0, 5.0), (0.0, 0.0, -5.0)],
        );

        assert_eq!(
            plan.debug_rects,
            vec![
                EffectSpawnerRectPlan {
                    x: 110.0,
                    y: 200.0,
                    width: 4.0,
                    height: 8.0,
                    rotation: 0.0,
                },
                EffectSpawnerRectPlan {
                    x: 90.0,
                    y: 200.0,
                    width: 4.0,
                    height: 8.0,
                    rotation: 0.0,
                },
            ]
        );
        assert_eq!(plan.spawns.len(), 1);
        assert_eq!(plan.spawns[0].effect, "sparkShoot");
        assert_eq!(plan.spawns[0].color, "white");
        assert!((plan.spawns[0].x - 110.0).abs() < 0.0001);
        assert!((plan.spawns[0].y - 200.0).abs() < 0.0001);
        assert_eq!(plan.spawns[0].rotation, 95.0);

        let paused = part.draw_plan(&params, 0.0, 1.0, true, &[0.0, 0.0], &[]);
        assert!(paused.spawns.is_empty());
        assert_eq!(paused.debug_rects.len(), 2);
    }

    #[test]
    fn shape_part_draw_plan_interpolates_progress_mirroring_and_shape_rotation() {
        let mut params = PartParams::default();
        params.set(0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 100.0, 200.0, 90.0);

        let part = ShapePart {
            mirror: true,
            sides: 4,
            radius: 3.0,
            radius_to: 9.0,
            stroke: 1.0,
            stroke_to: 3.0,
            x: 10.0,
            y: 2.0,
            rotation: 15.0,
            move_x: 4.0,
            move_y: 6.0,
            move_rot: 20.0,
            rotate_speed: 2.0,
            color: "from".into(),
            color_to: Some("to".into()),
            layer: 5.0,
            layer_offset: 0.25,
            ..Default::default()
        };

        let plan = part.draw_plan(&params, 3.0);
        assert_eq!(plan.layer, Some(5.0));
        assert_eq!(plan.layer_offset, 0.25);
        assert_eq!(plan.shapes.len(), 2);
        assert_eq!(plan.shapes[0].kind, ShapePartKind::Polygon { sides: 4 });
        assert_eq!(plan.shapes[0].radius, 6.0);
        assert_eq!(plan.shapes[0].stroke, 2.0);
        assert_eq!(plan.shapes[0].color, "from");
        assert_eq!(plan.shapes[0].color_to.as_deref(), Some("to"));
        assert_eq!(plan.shapes[0].color_mix, 0.5);
        assert_eq!((plan.shapes[0].x, plan.shapes[0].y), (112.0, 205.0));
        assert_eq!(plan.shapes[0].rotation, 31.0);
        assert_eq!((plan.shapes[1].x, plan.shapes[1].y), (88.0, 205.0));
        assert_eq!(plan.shapes[1].rotation, 149.0);

        let circle = ShapePart {
            circle: true,
            hollow: true,
            ..Default::default()
        }
        .draw_plan(&params, 0.0);
        assert_eq!(circle.shapes[0].kind, ShapePartKind::Circle);
        assert!(circle.shapes[0].hollow);
    }

    #[test]
    fn flare_part_draw_plan_emits_outer_and_inner_triangles() {
        let mut params = PartParams::default();
        params.set(0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 100.0, 200.0, 90.0);

        let part = FlarePart {
            sides: 2,
            radius: 10.0,
            radius_to: 20.0,
            stroke: 6.0,
            x: 5.0,
            y: 0.0,
            rotation: 10.0,
            rot_move: 20.0,
            spin_speed: 2.0,
            follow_rotation: true,
            ..Default::default()
        };

        let plan = part.draw_plan(&params, 3.0);
        assert_eq!((plan.x, plan.y), (105.0, 200.0));
        assert_eq!(plan.layer, 110.0);
        assert_eq!(plan.triangles.len(), 4);
        assert_eq!(plan.triangles[0].color, "techBlue");
        assert_eq!(plan.triangles[0].width, 6.0);
        assert_eq!(plan.triangles[0].length, 15.0);
        assert_eq!(plan.triangles[0].rotation, 116.0);
        assert_eq!(plan.triangles[1].rotation, 296.0);
        assert_eq!(plan.triangles[2].color, "white");
        assert_eq!(plan.triangles[2].width, 3.0);
        assert!((plan.triangles[2].length - 4.95).abs() < 0.0001);
    }
}
