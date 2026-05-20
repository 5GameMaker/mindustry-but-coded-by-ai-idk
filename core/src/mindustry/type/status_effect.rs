use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};

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
        Self {
            base: UnlockableContentBase::new(id, ContentType::Status, name),
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
}
