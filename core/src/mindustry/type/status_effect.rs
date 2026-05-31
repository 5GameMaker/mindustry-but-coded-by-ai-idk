use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};
use crate::mindustry::entities::units::StatusEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct StatusEffectFxPlan {
    pub effect: String,
    pub color_rgba: u32,
    pub parentize: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEffectRemovedPlan {
    pub effect: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusIntervalDamageKind {
    Normal,
    Pierce,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusIntervalDamagePlan {
    pub damage: f32,
    pub kind: StatusIntervalDamageKind,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatusEffectUpdatePlan {
    /// Mirrors Java `unit.damageContinuousPierce(damage)`. The receiving unit
    /// runtime is responsible for applying its usual continuous-damage delta.
    pub continuous_pierce_damage: f32,
    /// Mirrors Java `unit.heal(-damage * Time.delta)` for negative damage.
    pub heal: f32,
    pub interval_damage: Option<StatusIntervalDamagePlan>,
    pub visual_effect: Option<StatusEffectFxPlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusEffect {
    pub base: UnlockableContentBase,
    pub damage_multiplier: f32,
    pub health_multiplier: f32,
    pub speed_multiplier: f32,
    pub reload_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub drag_multiplier: f32,
    pub transition_damage: f32,
    pub disarm: bool,
    pub damage: f32,
    pub interval_damage_time: f32,
    pub interval_damage: f32,
    pub interval_damage_pierce: bool,
    pub effect_chance: f32,
    pub parentize_effect: bool,
    pub permanent: bool,
    pub reactive: bool,
    pub dynamic: bool,
    pub show: bool,
    pub color_rgba: u32,
    pub effect: String,
    pub apply_effect: String,
    pub apply_extend: bool,
    pub apply_color_rgba: u32,
    pub parentize_apply_effect: bool,
    pub opposites: Vec<String>,
    pub affinities: Vec<String>,
    pub outline: bool,
    pub transitions: Vec<String>,
}

impl StatusEffect {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        let mut base = UnlockableContentBase::new(id, ContentType::Status, name);
        base.all_database_tabs = true;
        Self {
            base,
            damage_multiplier: 1.0,
            health_multiplier: 1.0,
            speed_multiplier: 1.0,
            reload_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            drag_multiplier: 1.0,
            transition_damage: 0.0,
            disarm: false,
            damage: 0.0,
            interval_damage_time: 0.0,
            interval_damage: 0.0,
            interval_damage_pierce: false,
            effect_chance: 0.15,
            parentize_effect: false,
            permanent: false,
            reactive: false,
            dynamic: false,
            show: true,
            color_rgba: 0xffffffff,
            effect: "none".to_string(),
            apply_effect: "none".to_string(),
            apply_extend: false,
            apply_color_rgba: 0xffffffff,
            parentize_apply_effect: false,
            opposites: Vec::new(),
            affinities: Vec::new(),
            outline: true,
            transitions: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.base.mappable.name
    }

    pub fn localized_name(&self) -> &str {
        self.base
            .localized_name
            .as_deref()
            .unwrap_or_else(|| self.name())
    }

    pub fn content_type(&self) -> ContentType {
        ContentType::Status
    }

    pub fn is_hidden(&self) -> bool {
        self.localized_name() == self.name() || !self.show
    }

    pub fn show_unlock(&self) -> bool {
        false
    }

    pub fn add_transition(&mut self, effect_name: impl Into<String>) -> &mut Self {
        push_unique(&mut self.transitions, effect_name.into());
        self
    }

    pub fn add_affinity(&mut self, effect_name: impl Into<String>) -> &mut Self {
        let effect_name = effect_name.into();
        push_unique(&mut self.affinities, effect_name.clone());
        self.add_transition(effect_name)
    }

    pub fn add_opposite(&mut self, effect_name: impl Into<String>) -> &mut Self {
        let effect_name = effect_name.into();
        push_unique(&mut self.opposites, effect_name.clone());
        self.add_transition(effect_name)
    }

    pub fn reacts_with_name(&self, effect_name: &str) -> bool {
        self.transitions
            .iter()
            .any(|transition| transition == effect_name)
    }

    pub fn reacts_with(&self, effect: &StatusEffect) -> bool {
        self.reacts_with_name(effect.name())
    }

    pub fn update_plan(
        &self,
        entry: &mut StatusEntry,
        delta: f32,
        effect_roll: Option<f32>,
    ) -> StatusEffectUpdatePlan {
        let mut plan = StatusEffectUpdatePlan::default();

        if self.damage > 0.0 {
            plan.continuous_pierce_damage = self.damage;
        } else if self.damage < 0.0 {
            plan.heal = -self.damage * delta;
        }

        if self.interval_damage_time > 0.0 {
            entry.damage_time += delta;
            if entry.damage_time >= self.interval_damage_time {
                entry.damage_time %= self.interval_damage_time;
                plan.interval_damage = Some(StatusIntervalDamagePlan {
                    damage: self.interval_damage,
                    kind: if self.interval_damage_pierce {
                        StatusIntervalDamageKind::Pierce
                    } else {
                        StatusIntervalDamageKind::Normal
                    },
                });
            }
        }

        if self.effect != "none" {
            let chance = (self.effect_chance * delta).clamp(0.0, 1.0);
            if effect_roll.is_some_and(|roll| roll < chance) {
                plan.visual_effect = Some(StatusEffectFxPlan {
                    effect: self.effect.clone(),
                    color_rgba: self.color_rgba,
                    parentize: self.parentize_effect,
                });
            }
        }

        plan
    }

    pub fn applied_plan(&self, extend: bool) -> Option<StatusEffectFxPlan> {
        if self.apply_effect == "none" || (extend && !self.apply_extend) {
            return None;
        }

        Some(StatusEffectFxPlan {
            effect: self.apply_effect.clone(),
            color_rgba: self.apply_color_rgba,
            parentize: self.parentize_apply_effect,
        })
    }

    pub fn removed_plan(&self) -> StatusEffectRemovedPlan {
        StatusEffectRemovedPlan {
            effect: self.name().to_string(),
        }
    }
}

impl std::fmt::Display for StatusEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.localized_name())
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_effect_defaults_match_java_field_initializers() {
        let status = StatusEffect::new(5, "burning");

        assert_eq!(status.name(), "burning");
        assert_eq!(status.localized_name(), "burning");
        assert_eq!(status.content_type(), ContentType::Status);
        assert_eq!(status.damage_multiplier, 1.0);
        assert_eq!(status.health_multiplier, 1.0);
        assert_eq!(status.speed_multiplier, 1.0);
        assert_eq!(status.reload_multiplier, 1.0);
        assert_eq!(status.build_speed_multiplier, 1.0);
        assert_eq!(status.drag_multiplier, 1.0);
        assert_eq!(status.transition_damage, 0.0);
        assert!(!status.disarm);
        assert_eq!(status.damage, 0.0);
        assert_eq!(status.interval_damage_time, 0.0);
        assert_eq!(status.interval_damage, 0.0);
        assert!(!status.interval_damage_pierce);
        assert_eq!(status.effect_chance, 0.15);
        assert!(!status.parentize_effect);
        assert!(!status.permanent);
        assert!(!status.reactive);
        assert!(!status.dynamic);
        assert!(status.show);
        assert_eq!(status.color_rgba, 0xffffffff);
        assert_eq!(status.effect, "none");
        assert_eq!(status.apply_effect, "none");
        assert!(!status.apply_extend);
        assert_eq!(status.apply_color_rgba, 0xffffffff);
        assert!(!status.parentize_apply_effect);
        assert!(status.affinities.is_empty());
        assert!(status.opposites.is_empty());
        assert!(status.transitions.is_empty());
        assert!(status.outline);
        assert!(status.base.all_database_tabs);
        assert!(status.base.database_tabs.is_empty());
    }

    #[test]
    fn status_effect_hidden_display_and_unlock_follow_java_overrides() {
        let mut status = StatusEffect::new(1, "wet");
        assert!(status.is_hidden());
        assert!(!status.show_unlock());
        assert_eq!(status.to_string(), "wet");

        status.base.localized_name = Some("Wet".into());
        assert!(!status.is_hidden());
        assert_eq!(status.to_string(), "Wet");

        status.show = false;
        assert!(status.is_hidden());
    }

    #[test]
    fn status_effect_transition_queries_use_java_reacts_with_shape() {
        let mut wet = StatusEffect::new(1, "wet");
        let shocked = StatusEffect::new(2, "shocked");
        let tarred = StatusEffect::new(3, "tarred");

        assert!(!wet.reacts_with(&shocked));
        wet.add_affinity("shocked");
        wet.add_opposite("tarred");
        wet.add_transition("shocked");

        assert!(wet.reacts_with(&shocked));
        assert!(wet.reacts_with(&tarred));
        assert_eq!(wet.affinities, vec!["shocked"]);
        assert_eq!(wet.opposites, vec!["tarred"]);
        assert_eq!(wet.transitions, vec!["shocked", "tarred"]);
    }

    #[test]
    fn status_effect_update_plan_matches_java_damage_and_interval_timers() {
        let mut burning = StatusEffect::new(1, "burning");
        burning.damage = 0.25;
        burning.interval_damage_time = 5.0;
        burning.interval_damage = 12.0;
        burning.interval_damage_pierce = true;
        let mut entry = StatusEntry::new(burning.clone(), 60.0);

        let first = burning.update_plan(&mut entry, 3.0, None);
        assert_eq!(first.continuous_pierce_damage, 0.25);
        assert_eq!(first.heal, 0.0);
        assert_eq!(first.interval_damage, None);
        assert_eq!(entry.damage_time, 3.0);

        let second = burning.update_plan(&mut entry, 4.0, None);
        assert_eq!(
            second.interval_damage,
            Some(StatusIntervalDamagePlan {
                damage: 12.0,
                kind: StatusIntervalDamageKind::Pierce,
            })
        );
        assert_eq!(entry.damage_time, 2.0);

        let mut mending = StatusEffect::new(2, "mending");
        mending.damage = -0.5;
        let mut entry = StatusEntry::new(mending.clone(), 60.0);
        let plan = mending.update_plan(&mut entry, 6.0, None);
        assert_eq!(plan.continuous_pierce_damage, 0.0);
        assert_eq!(plan.heal, 3.0);
    }

    #[test]
    fn status_effect_update_and_apply_plans_gate_fx_like_java_hooks() {
        let mut status = StatusEffect::new(3, "wet");
        status.effect = "wetEffect".into();
        status.color_rgba = 0x3366ccff;
        status.parentize_effect = true;
        status.effect_chance = 0.2;
        let mut entry = StatusEntry::new(status.clone(), 60.0);

        assert_eq!(
            status
                .update_plan(&mut entry, 2.0, Some(0.39))
                .visual_effect,
            Some(StatusEffectFxPlan {
                effect: "wetEffect".into(),
                color_rgba: 0x3366ccff,
                parentize: true,
            })
        );
        assert_eq!(
            status.update_plan(&mut entry, 2.0, Some(0.4)).visual_effect,
            None
        );

        status.apply_effect = "applyWet".into();
        status.apply_color_rgba = 0x112233ff;
        status.parentize_apply_effect = true;
        assert_eq!(
            status.applied_plan(false),
            Some(StatusEffectFxPlan {
                effect: "applyWet".into(),
                color_rgba: 0x112233ff,
                parentize: true,
            })
        );
        assert_eq!(status.applied_plan(true), None);
        status.apply_extend = true;
        assert!(status.applied_plan(true).is_some());
    }

    #[test]
    fn status_effect_removed_plan_records_java_on_removed_hook_intent() {
        let status = StatusEffect::new(7, "melting");

        assert_eq!(
            status.removed_plan(),
            StatusEffectRemovedPlan {
                effect: "melting".into(),
            }
        );
    }
}
