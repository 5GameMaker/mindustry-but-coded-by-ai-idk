//! Status component mirroring upstream `mindustry.entities.comp.StatusComp`.

use std::collections::BTreeSet;

use crate::mindustry::entities::units::StatusEntry;
use crate::mindustry::r#type::status_effect::StatusEffectRemovedPlan;
use crate::mindustry::r#type::StatusEffect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl StatusColor {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusComp {
    pub statuses: Vec<StatusEntry>,
    pub applied: BTreeSet<String>,
    pub removed_plans: Vec<StatusEffectRemovedPlan>,
    pub speed_multiplier: f32,
    pub damage_multiplier: f32,
    pub health_multiplier: f32,
    pub reload_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub drag_multiplier: f32,
    pub armor_override: f32,
    pub disarmed: bool,
    pub immunities: BTreeSet<String>,
    pub type_speed: f32,
    pub type_build_speed: f32,
    pub type_drag: f32,
    pub max_health: f32,
}

impl StatusComp {
    pub fn new() -> Self {
        Self {
            statuses: Vec::with_capacity(4),
            applied: BTreeSet::new(),
            removed_plans: Vec::new(),
            speed_multiplier: 1.0,
            damage_multiplier: 1.0,
            health_multiplier: 1.0,
            reload_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            drag_multiplier: 1.0,
            armor_override: -1.0,
            disarmed: false,
            immunities: BTreeSet::new(),
            type_speed: 1.0,
            type_build_speed: 1.0,
            type_drag: 1.0,
            max_health: 1.0,
        }
    }

    pub fn apply_one_tick(&mut self, effect: StatusEffect) {
        self.apply(effect, 1.0);
    }

    pub fn apply(&mut self, effect: StatusEffect, duration: f32) {
        if effect.name() == "none" || self.is_immune(&effect) {
            return;
        }

        for entry in &mut self.statuses {
            if entry
                .effect
                .as_ref()
                .is_some_and(|existing| existing.name() == effect.name())
            {
                entry.time = entry.time.max(duration);
                return;
            }

            if entry
                .effect
                .as_ref()
                .is_some_and(|existing| existing.reacts_with(&effect))
            {
                Self::apply_transition(entry, effect, duration);
                return;
            }
        }

        if !effect.reactive {
            self.applied.insert(effect.name().to_string());
            self.statuses.push(StatusEntry::new(effect, duration));
        }
    }

    fn apply_transition(entry: &mut StatusEntry, incoming: StatusEffect, duration: f32) {
        let Some(existing) = entry.effect.clone() else {
            return;
        };
        let existing_name = existing.name();
        let incoming_name = incoming.name();

        if existing
            .opposites
            .iter()
            .any(|opposite| opposite == incoming_name)
        {
            entry.time -= duration * 0.5;
            if entry.time <= 0.0 {
                entry.set(incoming, duration);
            }
            return;
        }

        match (existing_name, incoming_name) {
            ("burning", "tarred") => {
                entry.time = (entry.time + duration).min(300.0);
            }
            ("melting", "tarred") => {
                entry.time = (entry.time + duration).min(200.0);
            }
            ("tarred", "burning") | ("tarred", "melting") => {
                entry.time += duration;
                entry.effect = Some(incoming);
            }
            ("freezing", "blasted") | ("wet", "shocked") => {
                // Java side effects damage, FX and campaign triggers here. The data-only
                // Rust port records the reaction by consuming the incoming reactive status
                // while keeping the current status entry unchanged.
            }
            _ => {}
        }
    }

    pub fn get_duration(&self, effect_name: &str) -> f32 {
        self.statuses
            .iter()
            .find(|entry| {
                entry
                    .effect
                    .as_ref()
                    .is_some_and(|effect| effect.name() == effect_name)
            })
            .map(|entry| entry.time)
            .unwrap_or(0.0)
    }

    pub fn clear_statuses(&mut self) {
        for entry in std::mem::take(&mut self.statuses) {
            self.record_removed_entry(&entry);
        }
        self.applied.clear();
    }

    pub fn unapply(&mut self, effect_name: &str) {
        let mut retained = Vec::with_capacity(self.statuses.len());
        for entry in std::mem::take(&mut self.statuses) {
            let remove = entry
                .effect
                .as_ref()
                .is_some_and(|effect| effect.name() == effect_name);
            if remove {
                self.record_removed_entry(&entry);
            } else {
                retained.push(entry);
            }
        }
        self.statuses = retained;
        self.applied.remove(effect_name);
    }

    pub fn is_boss(&self) -> bool {
        self.has_effect("boss")
    }

    pub fn is_immune(&self, effect: &StatusEffect) -> bool {
        self.immunities.contains(effect.name())
    }

    pub fn has_effect(&self, effect_name: &str) -> bool {
        self.statuses.iter().any(|entry| {
            entry
                .effect
                .as_ref()
                .is_some_and(|effect| effect.name() == effect_name)
        })
    }

    pub fn status_color(&self) -> StatusColor {
        if self.statuses.is_empty() {
            return StatusColor::WHITE;
        }

        let mut r = 1.0;
        let mut g = 1.0;
        let mut b = 1.0;
        let mut total = 0.0;
        for entry in &self.statuses {
            let Some(effect) = &entry.effect else {
                continue;
            };
            let intensity = if entry.time < 10.0 {
                entry.time / 10.0
            } else {
                1.0
            };
            let (er, eg, eb) = rgba_to_rgb(effect.color_rgba);
            r += er * intensity;
            g += eg * intensity;
            b += eb * intensity;
            total += intensity;
        }
        let count = self.statuses.len() as f32 + total;
        StatusColor {
            r: r / count,
            g: g / count,
            b: b / count,
            a: 1.0,
        }
    }

    pub fn apply_dynamic_status(&mut self) -> &mut StatusEntry {
        if let Some(index) = self.statuses.iter().position(|entry| {
            entry
                .effect
                .as_ref()
                .is_some_and(|effect| effect.dynamic || effect.name() == "dynamic")
        }) {
            return &mut self.statuses[index];
        }

        let mut dynamic = StatusEffect::new(-1, "dynamic");
        dynamic.dynamic = true;
        self.statuses.push(StatusEntry::new(dynamic, f32::INFINITY));
        self.applied.insert("dynamic".into());
        self.statuses.last_mut().unwrap()
    }

    pub fn status_speed(&mut self, speed: f32) {
        let type_speed = self.type_speed;
        self.apply_dynamic_status().speed_multiplier = speed / (type_speed * 60.0 / 8.0);
    }

    pub fn status_damage_multiplier(&mut self, damage_multiplier: f32) {
        self.apply_dynamic_status().damage_multiplier = damage_multiplier;
    }

    pub fn status_reload_multiplier(&mut self, reload_multiplier: f32) {
        self.apply_dynamic_status().reload_multiplier = reload_multiplier;
    }

    pub fn status_max_health(&mut self, health: f32) {
        let max_health = self.max_health;
        self.apply_dynamic_status().health_multiplier = health / max_health;
    }

    pub fn status_build_speed(&mut self, build_speed: f32) {
        let type_build_speed = self.type_build_speed;
        self.apply_dynamic_status().build_speed_multiplier = build_speed / type_build_speed;
    }

    pub fn status_drag(&mut self, drag: f32) {
        let type_drag = self.type_drag;
        self.apply_dynamic_status().drag_multiplier = if type_drag == 0.0 {
            0.0
        } else {
            drag / type_drag
        };
    }

    pub fn status_armor(&mut self, armor: f32) {
        self.apply_dynamic_status().armor_override = armor;
    }

    pub fn update(
        &mut self,
        delta: f32,
        floor_status: Option<(StatusEffect, f32)>,
        grounded: bool,
        hovering: bool,
    ) {
        if grounded && !hovering {
            if let Some((effect, duration)) = floor_status {
                self.apply(effect, duration);
            }
        }

        self.applied.clear();
        self.armor_override = -1.0;
        self.speed_multiplier = 1.0;
        self.damage_multiplier = 1.0;
        self.health_multiplier = 1.0;
        self.reload_multiplier = 1.0;
        self.build_speed_multiplier = 1.0;
        self.drag_multiplier = 1.0;
        self.disarmed = false;

        let mut retained = Vec::with_capacity(self.statuses.len());
        for mut entry in std::mem::take(&mut self.statuses) {
            entry.time = (entry.time - delta).max(0.0);
            let remove = entry
                .effect
                .as_ref()
                .map(|effect| entry.time <= 0.0 && !effect.permanent)
                .unwrap_or(true);
            if remove {
                self.record_removed_entry(&entry);
                continue;
            }

            if let Some(effect) = &entry.effect {
                self.applied.insert(effect.name().to_string());
                self.speed_multiplier *= effect.speed_multiplier * entry.speed_multiplier;
                self.health_multiplier *= effect.health_multiplier * entry.health_multiplier;
                self.damage_multiplier *= effect.damage_multiplier * entry.damage_multiplier;
                self.reload_multiplier *= effect.reload_multiplier * entry.reload_multiplier;
                self.build_speed_multiplier *=
                    effect.build_speed_multiplier * entry.build_speed_multiplier;
                self.drag_multiplier *= effect.drag_multiplier * entry.drag_multiplier;
                self.disarmed |= effect.disarm;
                if entry.armor_override >= 0.0 {
                    self.armor_override = entry.armor_override;
                }
            }

            retained.push(entry);
        }
        self.statuses = retained;
    }

    pub fn take_removed_plans(&mut self) -> Vec<StatusEffectRemovedPlan> {
        std::mem::take(&mut self.removed_plans)
    }

    fn record_removed_entry(&mut self, entry: &StatusEntry) {
        if let Some(effect) = &entry.effect {
            self.removed_plans.push(effect.removed_plan());
        }
    }
}

impl Default for StatusComp {
    fn default() -> Self {
        Self::new()
    }
}

fn rgba_to_rgb(rgba: u32) -> (f32, f32, f32) {
    (
        ((rgba >> 24) & 0xff) as f32 / 255.0,
        ((rgba >> 16) & 0xff) as f32 / 255.0,
        ((rgba >> 8) & 0xff) as f32 / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn effect(id: i16, name: &str, color: u32) -> StatusEffect {
        let mut effect = StatusEffect::new(id, name);
        effect.color_rgba = color;
        effect
    }

    fn vanilla_status(name: &str) -> StatusEffect {
        crate::mindustry::content::status_effects::load()
            .into_iter()
            .find(|status| status.name() == name)
            .unwrap_or_else(|| panic!("missing status effect: {name}"))
    }

    #[test]
    fn status_component_applies_extends_unapplies_and_checks_immunity() {
        let mut comp = StatusComp::new();
        comp.immunities.insert("wet".into());

        comp.apply(effect(1, "burning", 0xff0000ff), 5.0);
        comp.apply(effect(1, "burning", 0xff0000ff), 10.0);
        comp.apply(effect(2, "wet", 0x0000ffff), 10.0);

        assert_eq!(comp.statuses.len(), 1);
        assert_eq!(comp.get_duration("burning"), 10.0);
        assert_eq!(comp.get_duration("wet"), 0.0);
        assert!(comp.has_effect("burning"));

        comp.unapply("burning");
        assert!(!comp.has_effect("burning"));
    }

    #[test]
    fn status_component_dynamic_status_overrides_multipliers() {
        let mut comp = StatusComp::new();
        comp.type_speed = 2.0;
        comp.type_build_speed = 4.0;
        comp.type_drag = 5.0;
        comp.max_health = 100.0;

        comp.status_speed(30.0);
        comp.status_damage_multiplier(2.0);
        comp.status_reload_multiplier(3.0);
        comp.status_max_health(150.0);
        comp.status_build_speed(8.0);
        comp.status_drag(10.0);
        comp.status_armor(7.0);
        comp.update(0.0, None, false, false);

        assert_eq!(comp.speed_multiplier, 2.0);
        assert_eq!(comp.damage_multiplier, 2.0);
        assert_eq!(comp.reload_multiplier, 3.0);
        assert_eq!(comp.health_multiplier, 1.5);
        assert_eq!(comp.build_speed_multiplier, 2.0);
        assert_eq!(comp.drag_multiplier, 2.0);
        assert_eq!(comp.armor_override, 7.0);
    }

    #[test]
    fn status_component_update_decays_and_removes_nonpermanent_statuses() {
        let mut comp = StatusComp::new();
        comp.apply(effect(1, "burning", 0xff0000ff), 1.0);
        comp.update(2.0, None, false, false);

        assert!(comp.statuses.is_empty());
        assert!(comp.applied.is_empty());
        assert_eq!(
            comp.take_removed_plans(),
            vec![StatusEffectRemovedPlan {
                effect: "burning".into()
            }]
        );
    }

    #[test]
    fn status_component_floor_status_and_status_color_follow_java_shape() {
        let mut comp = StatusComp::new();
        comp.update(
            0.0,
            Some((effect(3, "muddy", 0x00ff00ff), 5.0)),
            true,
            false,
        );

        assert!(comp.has_effect("muddy"));
        let color = comp.status_color();
        assert!(color.g > color.r);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn status_component_opposite_transition_reduces_or_replaces_like_java_handler() {
        let mut comp = StatusComp::new();
        comp.apply(vanilla_status("burning"), 20.0);
        comp.apply(vanilla_status("wet"), 10.0);

        assert!(comp.has_effect("burning"));
        assert!(!comp.has_effect("wet"));
        assert_eq!(comp.get_duration("burning"), 15.0);

        comp.apply(vanilla_status("wet"), 40.0);
        assert!(!comp.has_effect("burning"));
        assert!(comp.has_effect("wet"));
        assert_eq!(comp.get_duration("wet"), 40.0);
    }

    #[test]
    fn status_component_affinity_transition_merges_tarred_fire_like_java_handlers() {
        let mut burning = StatusComp::new();
        burning.apply(vanilla_status("burning"), 250.0);
        burning.apply(vanilla_status("tarred"), 80.0);

        assert!(burning.has_effect("burning"));
        assert_eq!(burning.get_duration("burning"), 300.0);
        assert!(!burning.has_effect("tarred"));

        let mut tarred = StatusComp::new();
        tarred.apply(vanilla_status("tarred"), 12.0);
        tarred.apply(vanilla_status("melting"), 8.0);

        assert!(!tarred.has_effect("tarred"));
        assert!(tarred.has_effect("melting"));
        assert_eq!(tarred.get_duration("melting"), 20.0);
    }

    #[test]
    fn status_component_reactive_affinity_consumes_incoming_without_direct_application() {
        let mut freezing = StatusComp::new();
        freezing.apply(vanilla_status("freezing"), 30.0);
        freezing.apply(vanilla_status("blasted"), 20.0);

        assert!(freezing.has_effect("freezing"));
        assert_eq!(freezing.get_duration("freezing"), 30.0);
        assert!(!freezing.has_effect("blasted"));

        let mut wet = StatusComp::new();
        wet.apply(vanilla_status("wet"), 30.0);
        wet.apply(vanilla_status("shocked"), 20.0);

        assert!(wet.has_effect("wet"));
        assert_eq!(wet.get_duration("wet"), 30.0);
        assert!(!wet.has_effect("shocked"));
    }

    #[test]
    fn status_component_clear_statuses_records_removed_plans() {
        let mut comp = StatusComp::new();
        comp.apply(effect(9, "tarred", 0x996633ff), 30.0);
        comp.apply(effect(10, "wet", 0x0000ffff), 20.0);

        comp.clear_statuses();

        assert!(comp.statuses.is_empty());
        assert!(comp.applied.is_empty());
        assert_eq!(
            comp.take_removed_plans(),
            vec![
                StatusEffectRemovedPlan {
                    effect: "tarred".into()
                },
                StatusEffectRemovedPlan {
                    effect: "wet".into()
                },
            ]
        );
    }
}
