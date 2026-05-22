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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverCirclePlan {
    pub x: f32,
    pub y: f32,
    pub sides: i32,
    pub radius: f32,
    pub rotation: f32,
    pub stroke: f32,
    pub fin: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HoverPartDrawPlan {
    pub color: String,
    pub layer: Option<f32>,
    pub layer_offset: f32,
    pub under_turret_shading: bool,
    pub circles: Vec<HoverCirclePlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HoverPart {
    pub config: DrawPartConfig,
    pub radius: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub phase: f32,
    pub stroke: f32,
    pub min_stroke: f32,
    pub circles: i32,
    pub sides: i32,
    pub color: String,
    pub mirror: bool,
    pub layer: f32,
    pub layer_offset: f32,
}

impl Default for HoverPart {
    fn default() -> Self {
        Self {
            config: DrawPartConfig::default(),
            radius: 4.0,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            phase: 50.0,
            stroke: 3.0,
            min_stroke: 0.12,
            circles: 2,
            sides: 4,
            color: "white".into(),
            mirror: false,
            layer: -1.0,
            layer_offset: 0.0,
        }
    }
}

impl HoverPart {
    pub fn draw_plan(&self, params: &PartParams, time: f32) -> HoverPartDrawPlan {
        let len = if self.mirror && params.side_override == -1 {
            2
        } else {
            1
        };
        let mut circles = Vec::with_capacity(self.circles.max(0) as usize * len);
        for c in 0..self.circles.max(0) {
            let fin = (time / self.phase + c as f32 / self.circles.max(1) as f32).rem_euclid(1.0);
            let stroke = (1.0 - fin) * self.stroke + self.min_stroke;
            for side in 0..len {
                let i = if params.side_override == -1 {
                    side as i32
                } else {
                    params.side_override
                };
                let sign = (if i == 0 { 1.0 } else { -1.0 }) * params.side_multiplier as f32;
                let offset = rotate_offset(params.rotation - 90.0, self.x * sign, self.y);
                circles.push(HoverCirclePlan {
                    x: params.x + offset.0,
                    y: params.y + offset.1,
                    sides: self.sides,
                    radius: self.radius * fin,
                    rotation: params.rotation,
                    stroke,
                    fin,
                });
            }
        }

        HoverPartDrawPlan {
            color: self.color.clone(),
            layer: (self.layer > 0.0).then_some(self.layer),
            layer_offset: self.layer_offset,
            under_turret_shading: self.config.under && self.config.turret_shading,
            circles,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaloShapeKind {
    Triangle,
    Polygon { sides: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HaloShapePlan {
    pub x: f32,
    pub y: f32,
    pub kind: HaloShapeKind,
    pub radius: f32,
    pub length: f32,
    pub stroke: f32,
    pub rotation: f32,
    pub hollow: bool,
    pub color: String,
    pub color_to: Option<String>,
    pub color_mix: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HaloPartDrawPlan {
    pub layer: Option<f32>,
    pub layer_offset: f32,
    pub under_turret_shading: bool,
    pub shapes: Vec<HaloShapePlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HaloPart {
    pub config: DrawPartConfig,
    pub hollow: bool,
    pub tri: bool,
    pub shapes: i32,
    pub sides: i32,
    pub radius: f32,
    pub radius_to: f32,
    pub stroke: f32,
    pub stroke_to: f32,
    pub tri_length: f32,
    pub tri_length_to: f32,
    pub halo_radius: f32,
    pub halo_radius_to: f32,
    pub x: f32,
    pub y: f32,
    pub shape_rotation: f32,
    pub move_x: f32,
    pub move_y: f32,
    pub shape_move_rot: f32,
    pub halo_rotate_speed: f32,
    pub halo_rotation: f32,
    pub rotate_speed: f32,
    pub color: String,
    pub color_to: Option<String>,
    pub mirror: bool,
    pub clamp_progress: bool,
    pub progress: PartProgress,
    pub layer: f32,
    pub layer_offset: f32,
}

impl Default for HaloPart {
    fn default() -> Self {
        Self {
            config: DrawPartConfig::default(),
            hollow: false,
            tri: false,
            shapes: 3,
            sides: 3,
            radius: 3.0,
            radius_to: -1.0,
            stroke: 1.0,
            stroke_to: -1.0,
            tri_length: 1.0,
            tri_length_to: -1.0,
            halo_radius: 10.0,
            halo_radius_to: -1.0,
            x: 0.0,
            y: 0.0,
            shape_rotation: 0.0,
            move_x: 0.0,
            move_y: 0.0,
            shape_move_rot: 0.0,
            halo_rotate_speed: 0.0,
            halo_rotation: 0.0,
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

impl HaloPart {
    pub fn draw_plan(&self, params: &PartParams, time: f32) -> HaloPartDrawPlan {
        let prog = self.progress.get_clamp(params, time, self.clamp_progress);
        let base_rot = time * self.rotate_speed;
        let radius = if self.radius_to < 0.0 {
            self.radius
        } else {
            lerp(self.radius, self.radius_to, prog)
        };
        let tri_length = if self.tri_length_to < 0.0 {
            self.tri_length
        } else {
            lerp(self.tri_length, self.tri_length_to, prog)
        };
        let stroke = if self.stroke_to < 0.0 {
            self.stroke
        } else {
            lerp(self.stroke, self.stroke_to, prog)
        };
        let halo_radius = if self.halo_radius_to < 0.0 {
            self.halo_radius
        } else {
            lerp(self.halo_radius, self.halo_radius_to, prog)
        };
        let len = if self.mirror && params.side_override == -1 {
            2
        } else {
            1
        };
        let mut shapes = Vec::new();
        for side in 0..len {
            let i = if params.side_override == -1 {
                side as i32
            } else {
                params.side_override
            };
            let sign = (if i == 0 { 1.0 } else { -1.0 }) * params.side_multiplier as f32;
            let center_offset = rotate_offset(
                params.rotation - 90.0,
                (self.x + self.move_x * prog) * sign,
                self.y + self.move_y * prog,
            );
            let rx = params.x + center_offset.0;
            let ry = params.y + center_offset.1;
            let halo_rot = (self.halo_rotation + self.halo_rotate_speed * time) * sign;

            for v in 0..self.shapes.max(0) {
                let rot = halo_rot + v as f32 * 360.0 / self.shapes.max(1) as f32 + params.rotation;
                let shape_offset = rotate_offset(rot, halo_radius, 0.0);
                shapes.push(HaloShapePlan {
                    x: rx + shape_offset.0,
                    y: ry + shape_offset.1,
                    kind: if self.tri {
                        HaloShapeKind::Triangle
                    } else {
                        HaloShapeKind::Polygon { sides: self.sides }
                    },
                    radius,
                    length: tri_length,
                    stroke,
                    rotation: rot
                        + self.shape_move_rot * prog * sign
                        + self.shape_rotation * sign
                        + base_rot * sign,
                    hollow: self.hollow,
                    color: self.color.clone(),
                    color_to: self.color_to.clone(),
                    color_mix: prog,
                });
            }
        }

        HaloPartDrawPlan {
            layer: (self.layer > 0.0).then_some(self.layer),
            layer_offset: self.layer_offset,
            under_turret_shading: self.config.under && self.config.turret_shading,
            shapes,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTexture {
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub found: bool,
}

impl RegionTexture {
    pub fn found(name: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            scale: 1.0,
            found: true,
        }
    }

    pub fn missing(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            width: 0.0,
            height: 0.0,
            scale: 1.0,
            found: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPartLoadPlan {
    pub real_name: String,
    pub regions: Vec<String>,
    pub outlines: Vec<String>,
    pub heat: String,
    pub light: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionDrawKind {
    Outline,
    Region,
    Heat,
    HeatLight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDrawItem {
    pub kind: RegionDrawKind,
    pub region: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub rotation: f32,
    pub color: Option<String>,
    pub color_to: Option<String>,
    pub color_mix: f32,
    pub mix_color: Option<String>,
    pub mix_color_to: Option<String>,
    pub heat_alpha: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPartDrawPlan {
    pub layer: Option<f32>,
    pub layer_offset: f32,
    pub under_turret_shading: bool,
    pub x_scale: f32,
    pub y_scale: f32,
    pub items: Vec<RegionDrawItem>,
    pub child_params: Vec<PartParams>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPart {
    pub config: DrawPartConfig,
    pub suffix: String,
    pub name: Option<String>,
    pub mirror: bool,
    pub outline: bool,
    pub replace_outline: bool,
    pub draw_region: bool,
    pub heat_light: bool,
    pub clamp_progress: bool,
    pub progress: PartProgress,
    pub grow_progress: PartProgress,
    pub heat_progress: PartProgress,
    pub blending: String,
    pub layer: f32,
    pub layer_offset: f32,
    pub heat_layer_offset: f32,
    pub turret_heat_layer: f32,
    pub outline_layer_offset: f32,
    pub x: f32,
    pub y: f32,
    pub x_scl: f32,
    pub y_scl: f32,
    pub rotation: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub move_x: f32,
    pub move_y: f32,
    pub grow_x: f32,
    pub grow_y: f32,
    pub move_rot: f32,
    pub heat_light_opacity: f32,
    pub color: Option<String>,
    pub color_to: Option<String>,
    pub mix_color: Option<String>,
    pub mix_color_to: Option<String>,
    pub heat_color: String,
    pub moves: Vec<PartMove>,
}

impl Default for RegionPart {
    fn default() -> Self {
        Self {
            config: DrawPartConfig::default(),
            suffix: String::new(),
            name: None,
            mirror: false,
            outline: true,
            replace_outline: false,
            draw_region: true,
            heat_light: false,
            clamp_progress: true,
            progress: PartProgress::Warmup,
            grow_progress: PartProgress::Warmup,
            heat_progress: PartProgress::Heat,
            blending: "normal".into(),
            layer: -1.0,
            layer_offset: 0.0,
            heat_layer_offset: 1.0,
            turret_heat_layer: 110.0,
            outline_layer_offset: -0.001,
            x: 0.0,
            y: 0.0,
            x_scl: 1.0,
            y_scl: 1.0,
            rotation: 0.0,
            origin_x: 0.0,
            origin_y: 0.0,
            move_x: 0.0,
            move_y: 0.0,
            grow_x: 0.0,
            grow_y: 0.0,
            move_rot: 0.0,
            heat_light_opacity: 0.3,
            color: None,
            color_to: None,
            mix_color: None,
            mix_color_to: None,
            heat_color: "turretHeat".into(),
            moves: Vec::new(),
        }
    }
}

impl RegionPart {
    pub fn new(region: impl Into<String>) -> Self {
        Self {
            suffix: region.into(),
            ..Default::default()
        }
    }

    pub fn load_plan(&self, base_name: &str) -> RegionPartLoadPlan {
        let real_name = self
            .name
            .clone()
            .unwrap_or_else(|| format!("{base_name}{}", self.suffix));
        let (regions, outlines) = if self.draw_region {
            if self.mirror && self.config.turret_shading {
                (
                    vec![format!("{real_name}-r"), format!("{real_name}-l")],
                    vec![
                        format!("{real_name}-r-outline"),
                        format!("{real_name}-l-outline"),
                    ],
                )
            } else {
                (
                    vec![real_name.clone()],
                    vec![format!("{real_name}-outline")],
                )
            }
        } else {
            (Vec::new(), Vec::new())
        };

        RegionPartLoadPlan {
            real_name: real_name.clone(),
            regions,
            outlines,
            heat: format!("{real_name}-heat"),
            light: format!("{real_name}-light"),
        }
    }

    pub fn draw_plan(
        &self,
        params: &PartParams,
        time: f32,
        regions: &[RegionTexture],
        outlines: &[RegionTexture],
        heat: Option<&RegionTexture>,
        light: Option<&RegionTexture>,
    ) -> RegionPartDrawPlan {
        let prog = self.progress.get_clamp(params, time, self.clamp_progress);
        let scl_prog = self
            .grow_progress
            .get_clamp(params, time, self.clamp_progress);
        let mut mx = self.move_x * prog;
        let mut my = self.move_y * prog;
        let mut mr = self.move_rot * prog + self.rotation;
        let mut gx = self.grow_x * scl_prog;
        let mut gy = self.grow_y * scl_prog;

        for movement in &self.moves {
            let p = movement
                .progress
                .get_clamp(params, time, self.clamp_progress);
            mx += movement.x * p;
            my += movement.y * p;
            mr += movement.rot * p;
            gx += movement.gx * p;
            gy += movement.gy * p;
        }

        let len = if self.mirror && params.side_override == -1 {
            2
        } else {
            1
        };
        let x_scale = self.x_scl + gx;
        let y_scale = self.y_scl + gy;
        let mut items = Vec::new();
        let mut child_params = Vec::with_capacity(len);

        for side in 0..len {
            let i = if params.side_override == -1 {
                side as i32
            } else {
                params.side_override
            };
            let sign = (if i == 0 { 1.0 } else { -1.0 }) * params.side_multiplier as f32;
            let offset = rotate_offset(params.rotation - 90.0, (self.x + mx) * sign, self.y + my);
            let rx = params.x + offset.0;
            let ry = params.y + offset.1;
            let rot = mr * sign + params.rotation - 90.0;
            let index = (i.max(0) as usize).min(regions.len().saturating_sub(1));

            if self.outline && self.draw_region {
                if let Some(outline) = outlines.get(index) {
                    items.push(self.region_item(
                        RegionDrawKind::Outline,
                        outline,
                        rx,
                        ry,
                        rot,
                        x_scale * sign,
                        y_scale,
                        prog,
                        0.0,
                    ));
                }
            }

            if self.draw_region {
                if let Some(region) = regions.get(index).filter(|region| region.found) {
                    items.push(self.region_item(
                        RegionDrawKind::Region,
                        region,
                        rx,
                        ry,
                        rot,
                        x_scale * sign,
                        y_scale,
                        prog,
                        0.0,
                    ));
                }
            }

            if let Some(heat) = heat.filter(|heat| heat.found) {
                let hprog = self
                    .heat_progress
                    .get_clamp(params, time, self.clamp_progress);
                items.push(self.region_item(
                    RegionDrawKind::Heat,
                    heat,
                    rx,
                    ry,
                    rot,
                    x_scale * sign,
                    y_scale,
                    prog,
                    hprog,
                ));
                if self.heat_light {
                    let light_region = light.filter(|light| light.found).unwrap_or(heat);
                    items.push(self.region_item(
                        RegionDrawKind::HeatLight,
                        light_region,
                        rx,
                        ry,
                        rot,
                        x_scale * sign,
                        y_scale,
                        prog,
                        hprog * self.heat_light_opacity,
                    ));
                }
            }

            let mut child = PartParams::default();
            child.set(
                params.warmup,
                params.reload,
                params.smooth_reload,
                params.heat,
                params.recoil,
                params.charge,
                rx,
                ry,
                mr * sign + params.rotation,
            );
            child.side_multiplier = params.side_multiplier;
            child.life = params.life;
            child.side_override = i;
            child_params.push(child);
        }

        RegionPartDrawPlan {
            layer: (self.layer > 0.0).then_some(self.layer),
            layer_offset: self.layer_offset,
            under_turret_shading: self.config.under && self.config.turret_shading,
            x_scale,
            y_scale,
            items,
            child_params,
        }
    }

    fn region_item(
        &self,
        kind: RegionDrawKind,
        region: &RegionTexture,
        x: f32,
        y: f32,
        rotation: f32,
        x_scale: f32,
        y_scale: f32,
        color_mix: f32,
        heat_alpha: f32,
    ) -> RegionDrawItem {
        let width = region.width * region.scale * x_scale;
        let height = region.height * region.scale * y_scale;
        RegionDrawItem {
            kind,
            region: region.name.clone(),
            x,
            y,
            width,
            height,
            origin_x: width / 2.0 + self.origin_x * x_scale,
            origin_y: height / 2.0 + self.origin_y * y_scale,
            rotation,
            color: self.color.clone(),
            color_to: self.color_to.clone(),
            color_mix,
            mix_color: self.mix_color.clone(),
            mix_color_to: self.mix_color_to.clone(),
            heat_alpha,
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
        DrawPartConfig, EffectSpawnerPart, EffectSpawnerRectPlan, FlarePart, HaloPart,
        HaloShapeKind, HoverPart, PartMove, PartParams, PartProgress, RegionDrawKind, RegionPart,
        RegionTexture, ShapePart, ShapePartKind,
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

    #[test]
    fn hover_part_draw_plan_builds_phased_mirrored_polygons() {
        let mut params = PartParams::default();
        params.set(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 100.0, 200.0, 90.0);

        let part = HoverPart {
            radius: 10.0,
            x: 5.0,
            y: 0.0,
            phase: 20.0,
            stroke: 4.0,
            min_stroke: 0.5,
            circles: 2,
            sides: 6,
            mirror: true,
            layer: 7.0,
            layer_offset: 0.2,
            ..Default::default()
        };

        let plan = part.draw_plan(&params, 5.0);
        assert_eq!(plan.color, "white");
        assert_eq!(plan.layer, Some(7.0));
        assert_eq!(plan.layer_offset, 0.2);
        assert_eq!(plan.circles.len(), 4);
        assert_eq!((plan.circles[0].x, plan.circles[0].y), (105.0, 200.0));
        assert_eq!((plan.circles[1].x, plan.circles[1].y), (95.0, 200.0));
        assert_eq!(plan.circles[0].sides, 6);
        assert_eq!(plan.circles[0].fin, 0.25);
        assert_eq!(plan.circles[0].radius, 2.5);
        assert_eq!(plan.circles[0].stroke, 3.5);
        assert_eq!(plan.circles[2].fin, 0.75);
        assert_eq!(plan.circles[2].radius, 7.5);
        assert_eq!(plan.circles[2].stroke, 1.5);
    }

    #[test]
    fn halo_part_draw_plan_places_shapes_around_halo_radius() {
        let mut params = PartParams::default();
        params.set(0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 100.0, 200.0, 90.0);

        let part = HaloPart {
            tri: true,
            shapes: 2,
            radius: 3.0,
            radius_to: 5.0,
            tri_length: 2.0,
            tri_length_to: 6.0,
            halo_radius: 10.0,
            halo_radius_to: 20.0,
            x: 0.0,
            y: 0.0,
            shape_rotation: 10.0,
            shape_move_rot: 20.0,
            halo_rotation: 0.0,
            halo_rotate_speed: 0.0,
            rotate_speed: 2.0,
            color: "from".into(),
            color_to: Some("to".into()),
            ..Default::default()
        };

        let plan = part.draw_plan(&params, 3.0);
        assert_eq!(plan.shapes.len(), 2);
        assert_eq!(plan.shapes[0].kind, HaloShapeKind::Triangle);
        assert!((plan.shapes[0].x - 100.0).abs() < 0.0001);
        assert!((plan.shapes[0].y - 215.0).abs() < 0.0001);
        assert_eq!(plan.shapes[0].radius, 4.0);
        assert_eq!(plan.shapes[0].length, 4.0);
        assert_eq!(plan.shapes[0].rotation, 116.0);
        assert_eq!(plan.shapes[0].color, "from");
        assert_eq!(plan.shapes[0].color_to.as_deref(), Some("to"));
        assert_eq!(plan.shapes[0].color_mix, 0.5);
        assert!((plan.shapes[1].x - 100.0).abs() < 0.0001);
        assert!((plan.shapes[1].y - 185.0).abs() < 0.0001);
        assert_eq!(plan.shapes[1].rotation, 296.0);
    }

    #[test]
    fn region_part_load_plan_matches_mirrored_and_named_regions() {
        let mut part = RegionPart::new("-barrel");
        part.mirror = true;
        part.config.turret_shading = true;

        let plan = part.load_plan("duo");
        assert_eq!(plan.real_name, "duo-barrel");
        assert_eq!(plan.regions, vec!["duo-barrel-r", "duo-barrel-l"]);
        assert_eq!(
            plan.outlines,
            vec!["duo-barrel-r-outline", "duo-barrel-l-outline"]
        );
        assert_eq!(plan.heat, "duo-barrel-heat");
        assert_eq!(plan.light, "duo-barrel-light");

        part.name = Some("custom-region".into());
        part.draw_region = false;
        let heat_only = part.load_plan("ignored");
        assert_eq!(heat_only.real_name, "custom-region");
        assert!(heat_only.regions.is_empty());
        assert!(heat_only.outlines.is_empty());
        assert_eq!(heat_only.heat, "custom-region-heat");
        assert_eq!(heat_only.light, "custom-region-light");
    }

    #[test]
    fn region_part_draw_plan_builds_mirrored_region_heat_and_child_params() {
        let mut params = PartParams::default();
        params.set(0.5, 0.0, 0.0, 0.25, 0.0, 0.0, 100.0, 200.0, 90.0);
        params.life = 0.7;

        let part = RegionPart {
            mirror: true,
            heat_light: true,
            x: 10.0,
            y: 2.0,
            rotation: 15.0,
            move_x: 4.0,
            move_y: 6.0,
            move_rot: 20.0,
            color: Some("from".into()),
            color_to: Some("to".into()),
            mix_color: Some("mix-from".into()),
            mix_color_to: Some("mix-to".into()),
            ..Default::default()
        };

        let regions = [
            RegionTexture::found("barrel-r", 20.0, 10.0),
            RegionTexture::found("barrel-l", 20.0, 10.0),
        ];
        let outlines = [
            RegionTexture::found("barrel-r-outline", 22.0, 12.0),
            RegionTexture::found("barrel-l-outline", 22.0, 12.0),
        ];
        let heat = RegionTexture::found("barrel-heat", 20.0, 10.0);
        let light = RegionTexture::found("barrel-light", 24.0, 14.0);

        let plan = part.draw_plan(&params, 3.0, &regions, &outlines, Some(&heat), Some(&light));
        assert_eq!(plan.layer, None);
        assert_eq!(plan.layer_offset, 0.0);
        assert_eq!(plan.x_scale, 1.0);
        assert_eq!(plan.y_scale, 1.0);
        assert_eq!(plan.items.len(), 8);
        assert_eq!(plan.child_params.len(), 2);

        let outline = &plan.items[0];
        assert_eq!(outline.kind, RegionDrawKind::Outline);
        assert_eq!(outline.region, "barrel-r-outline");
        assert_eq!((outline.x, outline.y), (112.0, 205.0));
        assert_eq!((outline.width, outline.height), (22.0, 12.0));
        assert_eq!(outline.rotation, 25.0);
        assert_eq!(outline.color.as_deref(), Some("from"));
        assert_eq!(outline.color_to.as_deref(), Some("to"));
        assert_eq!(outline.color_mix, 0.5);

        let region = &plan.items[1];
        assert_eq!(region.kind, RegionDrawKind::Region);
        assert_eq!(region.region, "barrel-r");
        assert_eq!((region.width, region.height), (20.0, 10.0));
        assert_eq!(region.mix_color.as_deref(), Some("mix-from"));
        assert_eq!(region.mix_color_to.as_deref(), Some("mix-to"));

        let heat_item = &plan.items[2];
        assert_eq!(heat_item.kind, RegionDrawKind::Heat);
        assert_eq!(heat_item.region, "barrel-heat");
        assert_eq!(heat_item.heat_alpha, 0.25);

        let light_item = &plan.items[3];
        assert_eq!(light_item.kind, RegionDrawKind::HeatLight);
        assert_eq!(light_item.region, "barrel-light");
        assert!((light_item.heat_alpha - 0.075).abs() < 0.0001);

        let mirrored = &plan.items[5];
        assert_eq!(mirrored.kind, RegionDrawKind::Region);
        assert_eq!(mirrored.region, "barrel-l");
        assert_eq!((mirrored.x, mirrored.y), (88.0, 205.0));
        assert_eq!(mirrored.width, -20.0);
        assert_eq!(mirrored.height, 10.0);
        assert_eq!(mirrored.rotation, -25.0);

        assert_eq!(
            (
                plan.child_params[0].x,
                plan.child_params[0].y,
                plan.child_params[0].rotation,
                plan.child_params[0].side_override,
                plan.child_params[0].life,
            ),
            (112.0, 205.0, 115.0, 0, 0.7)
        );
        assert_eq!(
            (
                plan.child_params[1].x,
                plan.child_params[1].y,
                plan.child_params[1].rotation,
                plan.child_params[1].side_override,
            ),
            (88.0, 205.0, 65.0, 1)
        );
    }
}
