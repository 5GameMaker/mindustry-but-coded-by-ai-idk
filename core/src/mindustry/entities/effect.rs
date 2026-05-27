use crate::mindustry::{entities::comp::DecalColor, graphics::Layer};

pub const SHAKE_FALLOFF: f32 = 10000.0;
pub const DEFAULT_EFFECT_LIFETIME: f32 = 50.0;
pub const DEFAULT_EFFECT_CLIP: f32 = 50.0;
pub const DEFAULT_EFFECT_LAYER: f32 = 110.0;
/// Upstream `Fx.unitAssemble` id in `mindustry.content.Fx` for v158.1.
pub const FX_UNIT_ASSEMBLE_ID: i32 = 35;
/// Upstream `Fx.smoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_SMOKE_ID: i32 = 28;
/// Upstream `Fx.hitLiquid` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_LIQUID_ID: i32 = 85;
/// Upstream `Fx.missileTrail` id in `mindustry.content.Fx` for v158.1.
pub const FX_MISSILE_TRAIL_ID: i32 = 110;
/// Upstream `Fx.missileTrailShort` id in `mindustry.content.Fx` for v158.1.
pub const FX_MISSILE_TRAIL_SHORT_ID: i32 = 111;
/// Upstream `Fx.fire` id in `mindustry.content.Fx` for v158.1.
pub const FX_FIRE_ID: i32 = 119;
/// Upstream `Fx.fireSmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_FIRE_SMOKE_ID: i32 = 121;
/// Upstream `Fx.neoplasmHeal` id in `mindustry.content.Fx` for v158.1.
pub const FX_NEOPLASM_HEAL_ID: i32 = 122;
/// Upstream `Fx.steam` id in `mindustry.content.Fx` for v158.1.
pub const FX_STEAM_ID: i32 = 123;
/// Upstream `Fx.vapor` id in `mindustry.content.Fx` for v158.1.
pub const FX_VAPOR_ID: i32 = 128;
/// Upstream `Fx.fireballsmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_FIREBALL_SMOKE_ID: i32 = 130;
/// Upstream `Fx.smokeCloud` id in `mindustry.content.Fx` for v158.1.
pub const FX_SMOKE_CLOUD_ID: i32 = 222;
/// Upstream `Fx.ripple` id in `mindustry.content.Fx` for v158.1.
pub const FX_RIPPLE_ID: i32 = 243;

pub fn standard_effect_id(name: &str) -> Option<i32> {
    match name {
        "smoke" => Some(FX_SMOKE_ID),
        "hitLiquid" => Some(FX_HIT_LIQUID_ID),
        "unitAssemble" => Some(FX_UNIT_ASSEMBLE_ID),
        "missileTrail" => Some(FX_MISSILE_TRAIL_ID),
        "missileTrailShort" => Some(FX_MISSILE_TRAIL_SHORT_ID),
        "fire" => Some(FX_FIRE_ID),
        "fireSmoke" => Some(FX_FIRE_SMOKE_ID),
        "neoplasmHeal" => Some(FX_NEOPLASM_HEAL_ID),
        "steam" => Some(FX_STEAM_ID),
        "vapor" => Some(FX_VAPOR_ID),
        "fireballsmoke" => Some(FX_FIREBALL_SMOKE_ID),
        "smokeCloud" => Some(FX_SMOKE_CLOUD_ID),
        "ripple" => Some(FX_RIPPLE_ID),
        _ => None,
    }
}

pub fn standard_effect(effect_id: i32) -> Option<Effect> {
    let effect = match effect_id {
        FX_SMOKE_ID => Effect::with_lifetime(FX_SMOKE_ID, 100.0, DEFAULT_EFFECT_CLIP),
        FX_HIT_LIQUID_ID => Effect::with_lifetime(FX_HIT_LIQUID_ID, 16.0, DEFAULT_EFFECT_CLIP),
        FX_UNIT_ASSEMBLE_ID => {
            Effect::with_lifetime(FX_UNIT_ASSEMBLE_ID, 70.0, DEFAULT_EFFECT_CLIP)
                .layer(Layer::FLYING_UNIT + 5.0)
        }
        FX_MISSILE_TRAIL_ID => {
            Effect::with_lifetime(FX_MISSILE_TRAIL_ID, 50.0, DEFAULT_EFFECT_CLIP)
                .layer(Layer::BULLET - 0.001)
        }
        FX_MISSILE_TRAIL_SHORT_ID => {
            Effect::with_lifetime(FX_MISSILE_TRAIL_SHORT_ID, 22.0, DEFAULT_EFFECT_CLIP)
                .layer(Layer::BULLET - 0.001)
        }
        FX_FIRE_ID => Effect::with_lifetime(FX_FIRE_ID, 50.0, DEFAULT_EFFECT_CLIP),
        FX_FIRE_SMOKE_ID => Effect::with_lifetime(FX_FIRE_SMOKE_ID, 35.0, DEFAULT_EFFECT_CLIP),
        FX_NEOPLASM_HEAL_ID => {
            Effect::with_lifetime(FX_NEOPLASM_HEAL_ID, 120.0, DEFAULT_EFFECT_CLIP)
                .follow_parent(true)
                .rot_with_parent(true)
                .layer(Layer::BULLET - 2.0)
        }
        FX_STEAM_ID => Effect::with_lifetime(FX_STEAM_ID, 35.0, DEFAULT_EFFECT_CLIP),
        FX_VAPOR_ID => Effect::with_lifetime(FX_VAPOR_ID, 110.0, DEFAULT_EFFECT_CLIP),
        FX_FIREBALL_SMOKE_ID => {
            Effect::with_lifetime(FX_FIREBALL_SMOKE_ID, 25.0, DEFAULT_EFFECT_CLIP)
        }
        FX_SMOKE_CLOUD_ID => Effect::with_lifetime(FX_SMOKE_CLOUD_ID, 70.0, DEFAULT_EFFECT_CLIP),
        FX_RIPPLE_ID => {
            Effect::with_lifetime(FX_RIPPLE_ID, 30.0, DEFAULT_EFFECT_CLIP).layer(Layer::DEBRIS)
        }
        _ => return None,
    };
    Some(effect)
}

pub fn standard_effect_by_name(name: &str) -> Option<Effect> {
    standard_effect_id(name).and_then(standard_effect)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    pub id: i32,
    pub initialized: bool,
    pub lifetime: f32,
    pub clip: f32,
    pub start_delay: f32,
    pub base_rotation: f32,
    pub follow_parent: bool,
    pub rot_with_parent: bool,
    pub layer: f32,
    pub layer_duration: f32,
}

impl Effect {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            initialized: false,
            lifetime: DEFAULT_EFFECT_LIFETIME,
            clip: 0.0,
            start_delay: 0.0,
            base_rotation: 0.0,
            follow_parent: true,
            rot_with_parent: false,
            layer: DEFAULT_EFFECT_LAYER,
            layer_duration: 0.0,
        }
    }

    pub fn with_lifetime(id: i32, lifetime: f32, clip: f32) -> Self {
        Self {
            lifetime,
            clip,
            ..Self::new(id)
        }
    }

    pub fn start_delay(mut self, delay: f32) -> Self {
        self.start_delay = delay;
        self
    }

    pub fn follow_parent(mut self, follow: bool) -> Self {
        self.follow_parent = follow;
        self
    }

    pub fn rot_with_parent(mut self, follow: bool) -> Self {
        self.rot_with_parent = follow;
        self
    }

    pub fn layer(mut self, layer: f32) -> Self {
        self.layer = layer;
        self
    }

    pub fn layer_duration(mut self, layer: f32, duration: f32) -> Self {
        self.layer = layer;
        self.layer_duration = duration;
        self
    }

    pub fn base_rotation(mut self, rotation: f32) -> Self {
        self.base_rotation = rotation;
        self
    }

    pub fn should_create(&self, context: EffectCreateContext) -> bool {
        !context.headless && !context.is_none_effect && context.enable_effects
    }

    pub fn create_plan(
        &mut self,
        x: f32,
        y: f32,
        rotation: f32,
        color: DecalColor,
        data: Option<String>,
        parent: Option<EffectParent>,
        context: EffectCreateContext,
    ) -> Option<EffectCreatePlan> {
        if !self.should_create(context) || !context.camera_overlaps {
            return None;
        }

        let initialized_now = !self.initialized;
        self.initialized = true;

        let parent_id = parent
            .filter(|_| self.follow_parent)
            .map(|parent| parent.id);

        Some(EffectCreatePlan {
            delay: self.start_delay.max(0.0),
            initialized_now,
            spawn: EffectSpawnPlan {
                effect_id: self.id,
                x,
                y,
                rotation: self.base_rotation + rotation,
                color,
                data,
                lifetime: self.lifetime,
                clip: self.clip,
                layer: self.layer,
                layer_duration: self.layer_duration,
                parent_id,
                rot_with_parent: self.rot_with_parent && parent_id.is_some(),
            },
        })
    }

    pub fn render_with<F>(
        &self,
        input: EffectRenderParams,
        mut renderer: F,
    ) -> (EffectContainer, f32)
    where
        F: FnMut(&mut EffectContainer),
    {
        let mut container = EffectContainer::from_params(input);
        renderer(&mut container);
        let lifetime = container.lifetime;
        (container, lifetime)
    }
}

impl Default for Effect {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectParent {
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectCreateContext {
    pub headless: bool,
    pub enable_effects: bool,
    pub is_none_effect: bool,
    pub camera_overlaps: bool,
}

impl Default for EffectCreateContext {
    fn default() -> Self {
        Self {
            headless: false,
            enable_effects: true,
            is_none_effect: false,
            camera_overlaps: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectCreatePlan {
    pub delay: f32,
    pub initialized_now: bool,
    pub spawn: EffectSpawnPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectSpawnPlan {
    pub effect_id: i32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub color: DecalColor,
    pub data: Option<String>,
    pub lifetime: f32,
    pub clip: f32,
    pub layer: f32,
    pub layer_duration: f32,
    pub parent_id: Option<i32>,
    pub rot_with_parent: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectRenderParams {
    pub id: i32,
    pub color: DecalColor,
    pub time: f32,
    pub lifetime: f32,
    pub rotation: f32,
    pub x: f32,
    pub y: f32,
    pub data: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectContainer {
    pub x: f32,
    pub y: f32,
    pub time: f32,
    pub lifetime: f32,
    pub rotation: f32,
    pub color: DecalColor,
    pub id: i32,
    pub data: Option<String>,
}

impl EffectContainer {
    pub fn from_params(params: EffectRenderParams) -> Self {
        Self {
            x: params.x,
            y: params.y,
            color: params.color,
            time: params.time,
            lifetime: params.lifetime,
            id: params.id,
            rotation: params.rotation,
            data: params.data,
        }
    }

    pub fn fin(&self) -> f32 {
        self.time / self.lifetime
    }

    pub fn fout(&self) -> f32 {
        1.0 - self.fin()
    }

    pub fn finpow(&self) -> f32 {
        self.fin().powi(2)
    }

    pub fn scaled(&self, lifetime: f32) -> Option<Self> {
        (self.time <= lifetime).then(|| Self {
            lifetime,
            ..self.clone()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectRegistry {
    effects: Vec<Effect>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn create(&mut self, lifetime: f32, clip: f32) -> i32 {
        let id = self.effects.len() as i32;
        self.effects.push(Effect::with_lifetime(id, lifetime, clip));
        id
    }

    pub fn push(&mut self, mut effect: Effect) -> i32 {
        let id = self.effects.len() as i32;
        effect.id = id;
        self.effects.push(effect);
        id
    }

    pub fn get(&self, id: i32) -> Option<&Effect> {
        (id >= 0).then(|| self.effects.get(id as usize)).flatten()
    }

    pub fn get_mut(&mut self, id: i32) -> Option<&mut Effect> {
        if id >= 0 {
            self.effects.get_mut(id as usize)
        } else {
            None
        }
    }
}

impl Default for EffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiEffect {
    pub base: Effect,
    pub effects: Vec<Effect>,
}

impl Default for MultiEffect {
    fn default() -> Self {
        Self {
            base: Effect::default(),
            effects: Vec::new(),
        }
    }
}

impl MultiEffect {
    pub fn with_effects(effects: Vec<Effect>) -> Self {
        Self {
            effects,
            ..Default::default()
        }
    }

    pub fn create_plans(
        &mut self,
        x: f32,
        y: f32,
        rotation: f32,
        color: DecalColor,
        data: Option<String>,
        context: EffectCreateContext,
    ) -> Vec<EffectCreatePlan> {
        if !self.base.should_create(context) {
            return Vec::new();
        }

        self.effects
            .iter_mut()
            .filter_map(|effect| {
                effect.create_plan(x, y, rotation, color, data.clone(), None, context)
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeqRenderPlan {
    pub child_index: usize,
    pub params: EffectRenderParams,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeqEffect {
    pub base: Effect,
    pub effects: Vec<Effect>,
}

impl Default for SeqEffect {
    fn default() -> Self {
        let mut base = Effect::default();
        base.clip = 100.0;
        Self {
            base,
            effects: Vec::new(),
        }
    }
}

impl SeqEffect {
    pub fn with_effects(effects: Vec<Effect>) -> Self {
        Self {
            effects,
            ..Default::default()
        }
    }

    pub fn init_defaults(&mut self) {
        self.base.lifetime = 0.0;
        for effect in &self.effects {
            self.base.clip = self.base.clip.max(effect.clip);
            self.base.lifetime += effect.lifetime;
        }
    }

    pub fn render_plan(&mut self, input: EffectRenderParams) -> Option<SeqRenderPlan> {
        let mut sum = 0.0;
        for (index, effect) in self.effects.iter().enumerate() {
            if input.time <= effect.lifetime + sum {
                self.base.clip = self.base.clip.max(effect.clip);
                return Some(SeqRenderPlan {
                    child_index: index,
                    params: EffectRenderParams {
                        id: input.id + index as i32,
                        color: input.color,
                        time: input.time - sum,
                        lifetime: effect.lifetime,
                        rotation: input.rotation,
                        x: input.x,
                        y: input.y,
                        data: input.data,
                    },
                });
            }
            sum += effect.lifetime;
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WrapEffect {
    pub base: Effect,
    pub effect: Effect,
    pub color: DecalColor,
    pub rotation: f32,
}

impl Default for WrapEffect {
    fn default() -> Self {
        Self {
            base: Effect::default(),
            effect: Effect::default(),
            color: DecalColor::WHITE,
            rotation: 0.0,
        }
    }
}

impl WrapEffect {
    pub fn new(effect: Effect, color: DecalColor, rotation: f32) -> Self {
        Self {
            effect,
            color,
            rotation,
            ..Default::default()
        }
    }

    pub fn init_defaults(&mut self) {
        self.base.clip = self.effect.clip;
        self.base.lifetime = self.effect.lifetime;
    }

    pub fn create_plan(
        &mut self,
        x: f32,
        y: f32,
        data: Option<String>,
        context: EffectCreateContext,
    ) -> Option<EffectCreatePlan> {
        self.effect
            .create_plan(x, y, self.rotation, self.color, data, None, context)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialEffect {
    pub base: Effect,
    pub effect: Effect,
    pub rotation_spacing: f32,
    pub rotation_offset: f32,
    pub effect_rotation_offset: f32,
    pub length_offset: f32,
    pub amount: i32,
}

impl Default for RadialEffect {
    fn default() -> Self {
        let mut base = Effect::default();
        base.clip = 100.0;
        Self {
            base,
            effect: Effect::default(),
            rotation_spacing: 90.0,
            rotation_offset: 0.0,
            effect_rotation_offset: 0.0,
            length_offset: 0.0,
            amount: 4,
        }
    }
}

impl RadialEffect {
    pub fn new(
        effect: Effect,
        amount: i32,
        spacing: f32,
        length_offset: f32,
        effect_rotation_offset: f32,
    ) -> Self {
        Self {
            effect,
            amount,
            rotation_spacing: spacing,
            length_offset,
            effect_rotation_offset,
            ..Default::default()
        }
    }

    pub fn create_plans(
        &mut self,
        x: f32,
        y: f32,
        rotation: f32,
        color: DecalColor,
        data: Option<String>,
        context: EffectCreateContext,
    ) -> Vec<EffectCreatePlan> {
        if !self.base.should_create(context) {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(self.amount.max(0) as usize);
        let mut current_rotation = rotation + self.rotation_offset;
        for _ in 0..self.amount.max(0) {
            if let Some(plan) = self.effect.create_plan(
                x + trnsx(current_rotation, self.length_offset),
                y + trnsy(current_rotation, self.length_offset),
                current_rotation + self.effect_rotation_offset,
                color,
                data.clone(),
                None,
                context,
            ) {
                out.push(plan);
            }
            current_rotation += self.rotation_spacing;
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundPlaybackPlan {
    pub sound: String,
    pub x: f32,
    pub y: f32,
    pub delay: f32,
    pub pitch: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundEffectCreatePlan {
    pub sound: SoundPlaybackPlan,
    pub effect: Option<EffectCreatePlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundEffect {
    pub base: Effect,
    pub sound: String,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_volume: f32,
    pub max_volume: f32,
    pub effect: Effect,
}

impl Default for SoundEffect {
    fn default() -> Self {
        let mut base = Effect::default();
        base.start_delay = -1.0;
        Self {
            base,
            sound: "none".into(),
            min_pitch: 0.8,
            max_pitch: 1.2,
            min_volume: 1.0,
            max_volume: 1.0,
            effect: Effect::default(),
        }
    }
}

impl SoundEffect {
    pub fn new(sound: impl Into<String>, effect: Effect) -> Self {
        Self {
            sound: sound.into(),
            effect,
            ..Default::default()
        }
    }

    pub fn init_defaults(&mut self) {
        if self.base.start_delay < 0.0 {
            self.base.start_delay = self.effect.start_delay;
        }
    }

    pub fn create_plan(
        &mut self,
        x: f32,
        y: f32,
        rotation: f32,
        color: DecalColor,
        data: Option<String>,
        pitch_random: f32,
        volume_random: f32,
        context: EffectCreateContext,
    ) -> Option<SoundEffectCreatePlan> {
        if !self.base.should_create(context) {
            return None;
        }

        let pitch = lerp(self.min_pitch, self.max_pitch, pitch_random.clamp(0.0, 1.0));
        let volume = lerp(
            self.min_volume,
            self.max_volume,
            volume_random.clamp(0.0, 1.0),
        );
        Some(SoundEffectCreatePlan {
            sound: SoundPlaybackPlan {
                sound: self.sound.clone(),
                x,
                y,
                delay: self.base.start_delay.max(0.0),
                pitch,
                volume,
            },
            effect: self
                .effect
                .create_plan(x, y, rotation, color, data, None, context),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectInterp {
    Linear,
    Reverse,
}

impl EffectInterp {
    pub fn apply(self, from: f32, to: f32, t: f32) -> f32 {
        let t = match self {
            EffectInterp::Linear => t,
            EffectInterp::Reverse => 1.0 - t,
        }
        .clamp(0.0, 1.0);
        lerp(from, to, t)
    }

    pub fn scalar(self, t: f32) -> f32 {
        self.apply(0.0, 1.0, t)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaveDrawPlan {
    pub center: (f32, f32),
    pub color_from: String,
    pub color_to: String,
    pub color_mix: f32,
    pub stroke: f32,
    pub radius: f32,
    pub sides: i32,
    pub rotation: f32,
    pub light_radius: f32,
    pub light_color: String,
    pub light_opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaveEffect {
    pub base: Effect,
    pub color_from: String,
    pub color_to: String,
    pub light_color: Option<String>,
    pub size_from: f32,
    pub size_to: f32,
    pub light_scl: f32,
    pub light_opacity: f32,
    pub sides: i32,
    pub rotation: f32,
    pub stroke_from: f32,
    pub stroke_to: f32,
    pub interp: EffectInterp,
    pub light_interp: EffectInterp,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for WaveEffect {
    fn default() -> Self {
        Self {
            base: Effect::default(),
            color_from: "white".into(),
            color_to: "white".into(),
            light_color: None,
            size_from: 0.0,
            size_to: 100.0,
            light_scl: 3.0,
            light_opacity: 0.8,
            sides: -1,
            rotation: 0.0,
            stroke_from: 2.0,
            stroke_to: 0.0,
            interp: EffectInterp::Linear,
            light_interp: EffectInterp::Reverse,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

impl WaveEffect {
    pub fn init_defaults(&mut self) {
        self.base.clip = self
            .base
            .clip
            .max(self.size_from.max(self.size_to) + self.stroke_from.max(self.stroke_to));
    }

    pub fn draw_plan(&self, params: &EffectRenderParams) -> WaveDrawPlan {
        let fin = params.time / params.lifetime;
        let color_mix = self.interp.scalar(fin);
        let offset = rotate_offset(params.rotation, self.offset_x, self.offset_y);
        let center = (params.x + offset.0, params.y + offset.1);
        let radius = self.interp.apply(self.size_from, self.size_to, fin);
        WaveDrawPlan {
            center,
            color_from: self.color_from.clone(),
            color_to: self.color_to.clone(),
            color_mix,
            stroke: self.interp.apply(self.stroke_from, self.stroke_to, fin),
            radius,
            sides: self.sides,
            rotation: self.rotation + params.rotation,
            light_radius: radius * self.light_scl,
            light_color: self
                .light_color
                .clone()
                .unwrap_or_else(|| self.color_to.clone()),
            light_opacity: self.light_opacity * self.light_interp.scalar(fin),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionWavePlan {
    pub stroke: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionSmokePlan {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionSparkPlan {
    pub x: f32,
    pub y: f32,
    pub stroke: f32,
    pub angle: f32,
    pub length: f32,
    pub light_radius: f32,
    pub light_opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionDrawPlan {
    pub wave_color: String,
    pub smoke_color: String,
    pub spark_color: String,
    pub wave: Option<ExplosionWavePlan>,
    pub smoke_vector_radius: f32,
    pub spark_vector_radius: f32,
    pub smokes: Vec<ExplosionSmokePlan>,
    pub sparks: Vec<ExplosionSparkPlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionEffect {
    pub base: Effect,
    pub wave_color: String,
    pub smoke_color: String,
    pub spark_color: String,
    pub wave_life: f32,
    pub wave_stroke: f32,
    pub wave_rad: f32,
    pub wave_rad_base: f32,
    pub spark_stroke: f32,
    pub spark_rad: f32,
    pub spark_len: f32,
    pub smoke_size: f32,
    pub smoke_size_base: f32,
    pub smoke_rad: f32,
    pub smokes: i32,
    pub sparks: i32,
}

impl Default for ExplosionEffect {
    fn default() -> Self {
        let mut base = Effect::default();
        base.clip = 100.0;
        base.lifetime = 22.0;
        Self {
            base,
            wave_color: "missileYellow".into(),
            smoke_color: "gray".into(),
            spark_color: "missileYellowBack".into(),
            wave_life: 6.0,
            wave_stroke: 3.0,
            wave_rad: 15.0,
            wave_rad_base: 2.0,
            spark_stroke: 1.0,
            spark_rad: 23.0,
            spark_len: 3.0,
            smoke_size: 4.0,
            smoke_size_base: 0.5,
            smoke_rad: 23.0,
            smokes: 5,
            sparks: 4,
        }
    }
}

impl ExplosionEffect {
    pub fn draw_plan(
        &self,
        container: &EffectContainer,
        smoke_vectors: &[(f32, f32)],
        spark_vectors: &[(f32, f32)],
    ) -> ExplosionDrawPlan {
        let wave = container
            .scaled(self.wave_life)
            .map(|inner| ExplosionWavePlan {
                stroke: self.wave_stroke * inner.fout(),
                radius: self.wave_rad_base + inner.fin() * self.wave_rad,
            });
        let smoke_radius = container.fout() * self.smoke_size + self.smoke_size_base;
        let smokes = if self.smoke_size > 0.0 {
            smoke_vectors
                .iter()
                .take(self.smokes.max(0) as usize)
                .map(|(x, y)| ExplosionSmokePlan {
                    x: container.x + x,
                    y: container.y + y,
                    radius: smoke_radius,
                })
                .collect()
        } else {
            Vec::new()
        };

        let spark_stroke = container.fout() * self.spark_stroke;
        let spark_len = 1.0 + container.fout() * self.spark_len;
        let sparks = spark_vectors
            .iter()
            .take(self.sparks.max(0) as usize)
            .map(|(x, y)| ExplosionSparkPlan {
                x: container.x + x,
                y: container.y + y,
                stroke: spark_stroke,
                angle: (*y).atan2(*x).to_degrees(),
                length: spark_len,
                light_radius: container.fout() * self.spark_len * 4.0,
                light_opacity: 0.7,
            })
            .collect();

        ExplosionDrawPlan {
            wave_color: self.wave_color.clone(),
            smoke_color: self.smoke_color.clone(),
            spark_color: self.spark_color.clone(),
            wave,
            smoke_vector_radius: 2.0 + self.smoke_rad * container.finpow(),
            spark_vector_radius: 1.0 + self.spark_rad * container.finpow(),
            smokes,
            sparks,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleVectorInput {
    pub angle_offset: f32,
    pub length_factor: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticleDrawKind {
    Sprite {
        region: String,
        width: f32,
        height: f32,
        rotation: f32,
    },
    Line {
        stroke: f32,
        length: f32,
        angle: f32,
        cap: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleDrawItem {
    pub x: f32,
    pub y: f32,
    pub kind: ParticleDrawKind,
    pub light_radius: f32,
    pub light_color: String,
    pub light_opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleDrawPlan {
    pub color_from: String,
    pub color_to: String,
    pub color_mix: f32,
    pub origin: (f32, f32),
    pub requested_length: f32,
    pub particles: Vec<ParticleDrawItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleEffect {
    pub base: Effect,
    pub color_from: String,
    pub color_to: String,
    pub particles: i32,
    pub rand_length: bool,
    pub casing_flip: bool,
    pub cone: f32,
    pub length: f32,
    pub base_length: f32,
    pub interp: EffectInterp,
    pub size_interp: Option<EffectInterp>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub light_scl: f32,
    pub light_opacity: f32,
    pub light_color: Option<String>,
    pub spin: f32,
    pub size_from: f32,
    pub size_to: f32,
    pub size_change_start: f32,
    pub use_rotation: bool,
    pub offset: f32,
    pub region: String,
    pub line: bool,
    pub stroke_from: f32,
    pub stroke_to: f32,
    pub len_from: f32,
    pub len_to: f32,
    pub cap: bool,
}

impl Default for ParticleEffect {
    fn default() -> Self {
        Self {
            base: Effect::default(),
            color_from: "white".into(),
            color_to: "white".into(),
            particles: 6,
            rand_length: true,
            casing_flip: false,
            cone: 180.0,
            length: 20.0,
            base_length: 0.0,
            interp: EffectInterp::Linear,
            size_interp: None,
            offset_x: 0.0,
            offset_y: 0.0,
            light_scl: 2.0,
            light_opacity: 0.6,
            light_color: None,
            spin: 0.0,
            size_from: 2.0,
            size_to: 0.0,
            size_change_start: 0.0,
            use_rotation: true,
            offset: 0.0,
            region: "circle".into(),
            line: false,
            stroke_from: 2.0,
            stroke_to: 0.0,
            len_from: 4.0,
            len_to: 2.0,
            cap: true,
        }
    }
}

impl ParticleEffect {
    pub fn init_defaults(&mut self) {
        self.base.clip = self
            .base
            .clip
            .max(self.length + self.size_from.max(self.size_to));
        self.size_change_start = self.size_change_start.clamp(0.0, self.base.lifetime);
        if self.size_interp.is_none() {
            self.size_interp = Some(self.interp);
        }
    }

    pub fn draw_plan(
        &self,
        params: &EffectRenderParams,
        vectors: &[ParticleVectorInput],
        texture_ratio: f32,
    ) -> ParticleDrawPlan {
        let real_rotation = if self.use_rotation {
            if self.casing_flip {
                params.rotation.abs()
            } else {
                params.rotation
            }
        } else {
            self.base.base_rotation
        };
        let flip = if self.casing_flip {
            -signum_nonzero(params.rotation)
        } else {
            1.0
        };
        let raw_fin = params.time / params.lifetime;
        let fin = self.interp.scalar(raw_fin);
        let size_interp = self.size_interp.unwrap_or(self.interp);
        let size_curve = curve(raw_fin, self.size_change_start / params.lifetime, 1.0);
        let rad = size_interp.apply(self.size_from, self.size_to, size_curve) * 2.0;
        let offset = rotate_offset(real_rotation, self.offset_x * flip, self.offset_y);
        let origin = (params.x + offset.0, params.y + offset.1);
        let requested_length = self.length * fin + self.base_length;
        let light_color = self
            .light_color
            .clone()
            .unwrap_or_else(|| self.color_to.clone());

        let particles = vectors
            .iter()
            .take(self.particles.max(0) as usize)
            .map(|vector| {
                let len = if self.rand_length {
                    requested_length * vector.length_factor.clamp(0.0, 1.0)
                } else {
                    requested_length
                };
                let angle = real_rotation + vector.angle_offset.clamp(-self.cone, self.cone);
                let local = (trnsx(angle, len), trnsy(angle, len));
                let x = origin.0 + local.0;
                let y = origin.1 + local.1;
                if self.line {
                    let stroke = size_interp.apply(self.stroke_from, self.stroke_to, raw_fin);
                    let length = size_interp.apply(self.len_from, self.len_to, raw_fin);
                    ParticleDrawItem {
                        x,
                        y,
                        kind: ParticleDrawKind::Line {
                            stroke,
                            length,
                            angle: local.1.atan2(local.0).to_degrees(),
                            cap: self.cap,
                        },
                        light_radius: length * self.light_scl,
                        light_color: light_color.clone(),
                        light_opacity: self.light_opacity,
                    }
                } else {
                    ParticleDrawItem {
                        x,
                        y,
                        kind: ParticleDrawKind::Sprite {
                            region: self.region.clone(),
                            width: rad,
                            height: rad / texture_ratio.max(f32::EPSILON),
                            rotation: real_rotation + self.offset + params.time * self.spin,
                        },
                        light_radius: rad * self.light_scl,
                        light_color: light_color.clone(),
                        light_opacity: self.light_opacity,
                    }
                }
            })
            .collect();

        ParticleDrawPlan {
            color_from: self.color_from.clone(),
            color_to: self.color_to.clone(),
            color_mix: fin,
            origin,
            requested_length,
            particles,
        }
    }
}

fn trnsx(angle: f32, len: f32) -> f32 {
    angle.to_radians().cos() * len
}

fn trnsy(angle: f32, len: f32) -> f32 {
    angle.to_radians().sin() * len
}

fn rotate_offset(angle: f32, x: f32, y: f32) -> (f32, f32) {
    let rad = angle.to_radians();
    (x * rad.cos() - y * rad.sin(), x * rad.sin() + y * rad.cos())
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn curve(value: f32, start: f32, end: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return if value >= end { 1.0 } else { 0.0 };
    }
    ((value - start) / (end - start)).clamp(0.0, 1.0)
}

fn signum_nonzero(value: f32) -> f32 {
    if value < 0.0 {
        -1.0
    } else {
        1.0
    }
}

pub fn shake_intensity(intensity: f32, camera_x: f32, camera_y: f32, x: f32, y: f32) -> f32 {
    let dx = x - camera_x;
    let dy = y - camera_y;
    let mut distance = (dx * dx + dy * dy).sqrt();
    if distance < 1.0 {
        distance = 1.0;
    }

    (1.0 / (distance * distance / SHAKE_FALLOFF)).clamp(0.0, 1.0) * intensity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_effect_ids_include_puddle_ripple_dependencies() {
        assert_eq!(standard_effect_id("smoke"), Some(FX_SMOKE_ID));
        assert_eq!(standard_effect_id("hitLiquid"), Some(FX_HIT_LIQUID_ID));
        assert_eq!(
            standard_effect_id("unitAssemble"),
            Some(FX_UNIT_ASSEMBLE_ID)
        );
        assert_eq!(
            standard_effect_id("missileTrail"),
            Some(FX_MISSILE_TRAIL_ID)
        );
        assert_eq!(
            standard_effect_id("missileTrailShort"),
            Some(FX_MISSILE_TRAIL_SHORT_ID)
        );
        assert_eq!(standard_effect_id("fire"), Some(FX_FIRE_ID));
        assert_eq!(standard_effect_id("fireSmoke"), Some(FX_FIRE_SMOKE_ID));
        assert_eq!(
            standard_effect_id("neoplasmHeal"),
            Some(FX_NEOPLASM_HEAL_ID)
        );
        assert_eq!(standard_effect_id("steam"), Some(FX_STEAM_ID));
        assert_eq!(standard_effect_id("vapor"), Some(FX_VAPOR_ID));
        assert_eq!(
            standard_effect_id("fireballsmoke"),
            Some(FX_FIREBALL_SMOKE_ID)
        );
        assert_eq!(standard_effect_id("smokeCloud"), Some(FX_SMOKE_CLOUD_ID));
        assert_eq!(standard_effect_id("ripple"), Some(FX_RIPPLE_ID));
        assert_eq!(standard_effect_id("none"), None);
    }

    #[test]
    fn standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers() {
        let smoke = standard_effect_by_name("smoke").unwrap();
        assert_eq!(smoke.id, FX_SMOKE_ID);
        assert_eq!(smoke.lifetime, 100.0);
        assert_eq!(smoke.clip, 50.0);
        assert!(smoke.follow_parent);
        assert!(!smoke.rot_with_parent);

        let assemble = standard_effect(FX_UNIT_ASSEMBLE_ID).unwrap();
        assert_eq!(assemble.lifetime, 70.0);
        assert_eq!(assemble.clip, 50.0);
        assert_eq!(
            assemble.layer,
            crate::mindustry::graphics::Layer::FLYING_UNIT + 5.0
        );

        let trail = standard_effect(FX_MISSILE_TRAIL_SHORT_ID).unwrap();
        assert_eq!(trail.lifetime, 22.0);
        assert_eq!(
            trail.layer,
            crate::mindustry::graphics::Layer::BULLET - 0.001
        );

        let heal = standard_effect_by_name("neoplasmHeal").unwrap();
        assert_eq!(heal.lifetime, 120.0);
        assert!(heal.follow_parent);
        assert!(heal.rot_with_parent);
        assert_eq!(heal.layer, crate::mindustry::graphics::Layer::BULLET - 2.0);

        let ripple = standard_effect(FX_RIPPLE_ID).unwrap();
        assert_eq!(ripple.lifetime, 30.0);
        assert_eq!(ripple.layer, crate::mindustry::graphics::Layer::DEBRIS);
        assert!(standard_effect_by_name("none").is_none());
        assert!(standard_effect(-1).is_none());
    }

    #[test]
    fn effect_defaults_and_builder_methods_match_java_shape() {
        let effect = Effect::with_lifetime(3, 20.0, 40.0)
            .start_delay(5.0)
            .follow_parent(false)
            .rot_with_parent(true)
            .layer_duration(7.0, 9.0)
            .base_rotation(15.0);

        assert_eq!(effect.id, 3);
        assert_eq!(effect.lifetime, 20.0);
        assert_eq!(effect.clip, 40.0);
        assert_eq!(effect.start_delay, 5.0);
        assert!(!effect.follow_parent);
        assert!(effect.rot_with_parent);
        assert_eq!(effect.layer, 7.0);
        assert_eq!(effect.layer_duration, 9.0);
        assert_eq!(effect.base_rotation, 15.0);
    }

    #[test]
    fn create_plan_checks_headless_none_effects_camera_and_initializes_once() {
        let mut effect = Effect::with_lifetime(1, 30.0, 50.0).start_delay(2.0);

        assert!(effect
            .create_plan(
                1.0,
                2.0,
                3.0,
                DecalColor::WHITE,
                None,
                None,
                EffectCreateContext {
                    headless: true,
                    ..EffectCreateContext::default()
                },
            )
            .is_none());

        let plan = effect
            .create_plan(
                1.0,
                2.0,
                3.0,
                DecalColor::WHITE,
                Some("payload".into()),
                None,
                EffectCreateContext::default(),
            )
            .unwrap();

        assert!(plan.initialized_now);
        assert_eq!(plan.delay, 2.0);
        assert_eq!(plan.spawn.effect_id, 1);
        assert_eq!(plan.spawn.lifetime, 30.0);
        assert_eq!(plan.spawn.clip, 50.0);
        assert_eq!(plan.spawn.data.as_deref(), Some("payload"));

        let second = effect
            .create_plan(
                0.0,
                0.0,
                0.0,
                DecalColor::WHITE,
                None,
                None,
                EffectCreateContext::default(),
            )
            .unwrap();
        assert!(!second.initialized_now);

        assert!(effect
            .create_plan(
                0.0,
                0.0,
                0.0,
                DecalColor::WHITE,
                None,
                None,
                EffectCreateContext {
                    camera_overlaps: false,
                    ..EffectCreateContext::default()
                },
            )
            .is_none());
    }

    #[test]
    fn create_plan_applies_base_rotation_and_parent_flags() {
        let mut effect = Effect::with_lifetime(2, 10.0, 20.0)
            .base_rotation(30.0)
            .rot_with_parent(true);

        let plan = effect
            .create_plan(
                5.0,
                6.0,
                15.0,
                DecalColor::WHITE,
                None,
                Some(EffectParent { id: 99 }),
                EffectCreateContext::default(),
            )
            .unwrap();

        assert_eq!(plan.spawn.rotation, 45.0);
        assert_eq!(plan.spawn.parent_id, Some(99));
        assert!(plan.spawn.rot_with_parent);

        effect.follow_parent = false;
        let no_parent = effect
            .create_plan(
                5.0,
                6.0,
                15.0,
                DecalColor::WHITE,
                None,
                Some(EffectParent { id: 99 }),
                EffectCreateContext::default(),
            )
            .unwrap();
        assert_eq!(no_parent.spawn.parent_id, None);
        assert!(!no_parent.spawn.rot_with_parent);
    }

    #[test]
    fn effect_container_fin_scaled_and_render_lifetime_are_data_only() {
        let effect = Effect::with_lifetime(3, 10.0, 20.0);
        let params = EffectRenderParams {
            id: 7,
            color: DecalColor::WHITE,
            time: 5.0,
            lifetime: 10.0,
            rotation: 90.0,
            x: 1.0,
            y: 2.0,
            data: Some("data".into()),
        };

        let (container, lifetime) = effect.render_with(params, |container| {
            assert_eq!(container.fin(), 0.5);
            container.lifetime = 12.0;
        });

        assert_eq!(lifetime, 12.0);
        assert_eq!(container.scaled(6.0).unwrap().lifetime, 6.0);
        assert!(container.scaled(4.0).is_none());
    }

    #[test]
    fn effect_registry_assigns_ids_and_get_handles_invalid_ids() {
        let mut registry = EffectRegistry::new();

        assert_eq!(registry.create(10.0, 20.0), 0);
        assert_eq!(registry.push(Effect::with_lifetime(99, 30.0, 40.0)), 1);
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get(0).unwrap().id, 0);
        assert_eq!(registry.get(1).unwrap().id, 1);
        assert!(registry.get(-1).is_none());
        assert!(registry.get(99).is_none());
    }

    #[test]
    fn multi_effect_creates_all_child_effects_without_rendering_itself() {
        let child_a = Effect::with_lifetime(1, 10.0, 20.0).start_delay(2.0);
        let child_b = Effect::with_lifetime(2, 30.0, 40.0).base_rotation(5.0);
        let mut multi = MultiEffect::with_effects(vec![child_a, child_b]);

        let plans = multi.create_plans(
            7.0,
            8.0,
            9.0,
            DecalColor::WHITE,
            Some("payload".into()),
            EffectCreateContext::default(),
        );

        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].delay, 2.0);
        assert_eq!(plans[0].spawn.effect_id, 1);
        assert_eq!(plans[0].spawn.x, 7.0);
        assert_eq!(plans[0].spawn.y, 8.0);
        assert_eq!(plans[0].spawn.rotation, 9.0);
        assert_eq!(plans[0].spawn.data.as_deref(), Some("payload"));
        assert_eq!(plans[1].spawn.effect_id, 2);
        assert_eq!(plans[1].spawn.rotation, 14.0);
        assert!(plans[0].initialized_now);
        assert!(plans[1].initialized_now);

        let blocked = multi.create_plans(
            0.0,
            0.0,
            0.0,
            DecalColor::WHITE,
            None,
            EffectCreateContext {
                enable_effects: false,
                ..EffectCreateContext::default()
            },
        );
        assert!(blocked.is_empty());
    }

    #[test]
    fn seq_effect_sums_lifetime_clip_and_selects_child_by_time() {
        let child_a = Effect::with_lifetime(1, 10.0, 20.0);
        let child_b = Effect::with_lifetime(2, 30.0, 140.0);
        let mut seq = SeqEffect::with_effects(vec![child_a, child_b]);
        assert_eq!(seq.base.clip, 100.0);

        seq.init_defaults();
        assert_eq!(seq.base.lifetime, 40.0);
        assert_eq!(seq.base.clip, 140.0);

        let first = seq
            .render_plan(EffectRenderParams {
                id: 5,
                color: DecalColor::WHITE,
                time: 8.0,
                lifetime: 40.0,
                rotation: 45.0,
                x: 1.0,
                y: 2.0,
                data: Some("seq".into()),
            })
            .expect("first child should render");
        assert_eq!(first.child_index, 0);
        assert_eq!(first.params.id, 5);
        assert_eq!(first.params.time, 8.0);
        assert_eq!(first.params.lifetime, 10.0);
        assert_eq!(first.params.data.as_deref(), Some("seq"));

        let second = seq
            .render_plan(EffectRenderParams {
                id: 5,
                color: DecalColor::WHITE,
                time: 12.0,
                lifetime: 40.0,
                rotation: 45.0,
                x: 1.0,
                y: 2.0,
                data: None,
            })
            .expect("second child should render");
        assert_eq!(second.child_index, 1);
        assert_eq!(second.params.id, 6);
        assert_eq!(second.params.time, 2.0);
        assert_eq!(second.params.lifetime, 30.0);

        assert!(seq
            .render_plan(EffectRenderParams {
                id: 5,
                color: DecalColor::WHITE,
                time: 45.0,
                lifetime: 40.0,
                rotation: 0.0,
                x: 0.0,
                y: 0.0,
                data: None,
            })
            .is_none());
    }

    #[test]
    fn wrap_effect_syncs_child_lifetime_and_forwards_fixed_color_rotation() {
        let child = Effect::with_lifetime(4, 33.0, 77.0).base_rotation(5.0);
        let color = DecalColor {
            r: 0.2,
            g: 0.4,
            b: 0.6,
            a: 0.8,
        };
        let mut wrap = WrapEffect::new(child, color, 90.0);

        wrap.init_defaults();
        assert_eq!(wrap.base.lifetime, 33.0);
        assert_eq!(wrap.base.clip, 77.0);

        let plan = wrap
            .create_plan(
                3.0,
                4.0,
                Some("wrapped".into()),
                EffectCreateContext::default(),
            )
            .expect("wrapped child should create");
        assert_eq!(plan.spawn.effect_id, 4);
        assert_eq!(plan.spawn.x, 3.0);
        assert_eq!(plan.spawn.y, 4.0);
        assert_eq!(plan.spawn.rotation, 95.0);
        assert_eq!(plan.spawn.color, color);
        assert_eq!(plan.spawn.data.as_deref(), Some("wrapped"));
    }

    #[test]
    fn radial_effect_repeats_child_create_at_angle_intervals() {
        let child = Effect::with_lifetime(9, 10.0, 20.0);
        let mut radial = RadialEffect::new(child, 4, 90.0, 10.0, 5.0);
        radial.rotation_offset = 0.0;

        let plans = radial.create_plans(
            1.0,
            2.0,
            0.0,
            DecalColor::WHITE,
            Some("radial".into()),
            EffectCreateContext::default(),
        );

        assert_eq!(plans.len(), 4);
        assert_eq!(plans[0].spawn.effect_id, 9);
        assert!((plans[0].spawn.x - 11.0).abs() < 0.0001);
        assert!((plans[0].spawn.y - 2.0).abs() < 0.0001);
        assert_eq!(plans[0].spawn.rotation, 5.0);
        assert!((plans[1].spawn.x - 1.0).abs() < 0.0001);
        assert!((plans[1].spawn.y - 12.0).abs() < 0.0001);
        assert_eq!(plans[1].spawn.rotation, 95.0);
        assert!((plans[2].spawn.x + 9.0).abs() < 0.0001);
        assert!((plans[2].spawn.y - 2.0).abs() < 0.0001);
        assert_eq!(plans[2].spawn.rotation, 185.0);
        assert!((plans[3].spawn.x - 1.0).abs() < 0.0001);
        assert!((plans[3].spawn.y + 8.0).abs() < 0.0001);
        assert_eq!(plans[3].spawn.rotation, 275.0);
        assert_eq!(plans[3].spawn.data.as_deref(), Some("radial"));

        radial.amount = 0;
        assert!(radial
            .create_plans(
                0.0,
                0.0,
                0.0,
                DecalColor::WHITE,
                None,
                EffectCreateContext::default(),
            )
            .is_empty());
    }

    #[test]
    fn sound_effect_inherits_delay_and_records_sound_plus_child_effect() {
        let child = Effect::with_lifetime(12, 20.0, 30.0).start_delay(3.0);
        let mut sound = SoundEffect::new("boom", child);
        sound.min_pitch = 0.5;
        sound.max_pitch = 1.5;
        sound.min_volume = 0.25;
        sound.max_volume = 0.75;

        assert_eq!(sound.base.start_delay, -1.0);
        sound.init_defaults();
        assert_eq!(sound.base.start_delay, 3.0);

        let plan = sound
            .create_plan(
                4.0,
                5.0,
                6.0,
                DecalColor::WHITE,
                Some("sound".into()),
                0.25,
                0.5,
                EffectCreateContext::default(),
            )
            .expect("sound effect should create");
        assert_eq!(plan.sound.sound, "boom");
        assert_eq!(plan.sound.x, 4.0);
        assert_eq!(plan.sound.y, 5.0);
        assert_eq!(plan.sound.delay, 3.0);
        assert_eq!(plan.sound.pitch, 0.75);
        assert_eq!(plan.sound.volume, 0.5);

        let child = plan.effect.expect("child effect should also create");
        assert_eq!(child.spawn.effect_id, 12);
        assert_eq!(child.spawn.rotation, 6.0);
        assert_eq!(child.spawn.data.as_deref(), Some("sound"));

        assert!(sound
            .create_plan(
                0.0,
                0.0,
                0.0,
                DecalColor::WHITE,
                None,
                0.0,
                0.0,
                EffectCreateContext {
                    headless: true,
                    ..EffectCreateContext::default()
                },
            )
            .is_none());
    }

    #[test]
    fn wave_effect_init_and_draw_plan_follow_java_radius_stroke_light_math() {
        let mut wave = WaveEffect::default();
        wave.color_from = "from".into();
        wave.color_to = "to".into();
        wave.light_color = Some("light".into());
        wave.sides = 6;
        wave.rotation = 15.0;
        wave.offset_x = 10.0;
        wave.offset_y = 0.0;

        wave.init_defaults();
        assert_eq!(wave.base.clip, 102.0);

        let draw = wave.draw_plan(&EffectRenderParams {
            id: 1,
            color: DecalColor::WHITE,
            time: 25.0,
            lifetime: 100.0,
            rotation: 90.0,
            x: 1.0,
            y: 2.0,
            data: None,
        });
        assert!((draw.center.0 - 1.0).abs() < 0.0001);
        assert!((draw.center.1 - 12.0).abs() < 0.0001);
        assert_eq!(draw.color_from, "from");
        assert_eq!(draw.color_to, "to");
        assert_eq!(draw.color_mix, 0.25);
        assert_eq!(draw.stroke, 1.5);
        assert_eq!(draw.radius, 25.0);
        assert_eq!(draw.sides, 6);
        assert_eq!(draw.rotation, 105.0);
        assert_eq!(draw.light_radius, 75.0);
        assert_eq!(draw.light_color, "light");
        assert_eq!(draw.light_opacity, 0.6);
    }

    #[test]
    fn explosion_effect_draw_plan_covers_wave_smoke_and_sparks() {
        let explosion = ExplosionEffect::default();
        assert_eq!(explosion.base.clip, 100.0);
        assert_eq!(explosion.base.lifetime, 22.0);
        assert_eq!(explosion.wave_color, "missileYellow");
        assert_eq!(explosion.smoke_color, "gray");
        assert_eq!(explosion.spark_color, "missileYellowBack");
        assert_eq!(explosion.wave_life, 6.0);
        assert_eq!(explosion.smokes, 5);
        assert_eq!(explosion.sparks, 4);

        let container = EffectContainer {
            x: 10.0,
            y: 20.0,
            time: 0.0,
            lifetime: 22.0,
            rotation: 0.0,
            color: DecalColor::WHITE,
            id: 7,
            data: None,
        };
        let plan = explosion.draw_plan(&container, &[(1.0, 0.0), (0.0, 2.0)], &[(3.0, 4.0)]);
        assert_eq!(
            plan.wave,
            Some(ExplosionWavePlan {
                stroke: 3.0,
                radius: 2.0,
            })
        );
        assert_eq!(plan.smoke_vector_radius, 2.0);
        assert_eq!(plan.spark_vector_radius, 1.0);
        assert_eq!(
            plan.smokes,
            vec![
                ExplosionSmokePlan {
                    x: 11.0,
                    y: 20.0,
                    radius: 4.5,
                },
                ExplosionSmokePlan {
                    x: 10.0,
                    y: 22.0,
                    radius: 4.5,
                },
            ]
        );
        assert_eq!(plan.sparks.len(), 1);
        assert_eq!(plan.sparks[0].x, 13.0);
        assert_eq!(plan.sparks[0].y, 24.0);
        assert_eq!(plan.sparks[0].stroke, 1.0);
        assert!((plan.sparks[0].angle - 53.130104).abs() < 0.0001);
        assert_eq!(plan.sparks[0].length, 4.0);
        assert_eq!(plan.sparks[0].light_radius, 12.0);
        assert_eq!(plan.sparks[0].light_opacity, 0.7);
    }

    #[test]
    fn particle_effect_init_and_draw_plan_cover_sprite_and_line_modes() {
        let mut particle = ParticleEffect::default();
        assert_eq!(particle.color_from, "white");
        assert_eq!(particle.color_to, "white");
        assert_eq!(particle.particles, 6);
        assert!(particle.rand_length);
        assert_eq!(particle.cone, 180.0);
        assert_eq!(particle.length, 20.0);
        assert_eq!(particle.light_scl, 2.0);
        assert_eq!(particle.size_from, 2.0);
        assert_eq!(particle.size_to, 0.0);
        assert_eq!(particle.region, "circle");
        assert!(!particle.line);
        particle.init_defaults();
        assert_eq!(particle.base.clip, 22.0);
        assert_eq!(particle.size_interp, Some(EffectInterp::Linear));

        let params = EffectRenderParams {
            id: 1,
            color: DecalColor::WHITE,
            time: 25.0,
            lifetime: 50.0,
            rotation: 30.0,
            x: 0.0,
            y: 0.0,
            data: None,
        };
        let sprite = particle.draw_plan(
            &params,
            &[ParticleVectorInput {
                angle_offset: 0.0,
                length_factor: 1.0,
            }],
            2.0,
        );
        assert_eq!(sprite.color_mix, 0.5);
        assert_eq!(sprite.requested_length, 10.0);
        assert!((sprite.particles[0].x - 8.660254).abs() < 0.0001);
        assert!((sprite.particles[0].y - 5.0).abs() < 0.0001);
        assert_eq!(
            sprite.particles[0].kind,
            ParticleDrawKind::Sprite {
                region: "circle".into(),
                width: 2.0,
                height: 1.0,
                rotation: 30.0,
            }
        );
        assert_eq!(sprite.particles[0].light_radius, 4.0);
        assert_eq!(sprite.particles[0].light_opacity, 0.6);

        particle.line = true;
        particle.rand_length = false;
        particle.use_rotation = false;
        particle.base.base_rotation = 90.0;
        let line = particle.draw_plan(
            &params,
            &[ParticleVectorInput {
                angle_offset: 0.0,
                length_factor: 0.25,
            }],
            1.0,
        );
        assert!((line.particles[0].x).abs() < 0.0001);
        assert!((line.particles[0].y - 10.0).abs() < 0.0001);
        assert_eq!(
            line.particles[0].kind,
            ParticleDrawKind::Line {
                stroke: 1.0,
                length: 3.0,
                angle: 90.0,
                cap: true,
            }
        );
        assert_eq!(line.particles[0].light_radius, 6.0);
    }

    #[test]
    fn shake_intensity_falls_off_with_camera_distance() {
        assert_eq!(shake_intensity(5.0, 0.0, 0.0, 0.0, 0.0), 5.0);
        assert_eq!(shake_intensity(5.0, 0.0, 0.0, 100.0, 0.0), 5.0);
        assert_eq!(shake_intensity(8.0, 0.0, 0.0, 200.0, 0.0), 2.0);
    }
}
