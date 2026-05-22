use crate::mindustry::entities::comp::DecalColor;

pub const SHAKE_FALLOFF: f32 = 10000.0;
pub const DEFAULT_EFFECT_LIFETIME: f32 = 50.0;
pub const DEFAULT_EFFECT_CLIP: f32 = 50.0;
pub const DEFAULT_EFFECT_LAYER: f32 = 110.0;

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

fn trnsx(angle: f32, len: f32) -> f32 {
    angle.to_radians().cos() * len
}

fn trnsy(angle: f32, len: f32) -> f32 {
    angle.to_radians().sin() * len
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
    fn shake_intensity_falls_off_with_camera_distance() {
        assert_eq!(shake_intensity(5.0, 0.0, 0.0, 0.0, 0.0), 5.0);
        assert_eq!(shake_intensity(5.0, 0.0, 0.0, 100.0, 0.0), 5.0);
        assert_eq!(shake_intensity(8.0, 0.0, 0.0, 200.0, 0.0), 2.0);
    }
}
