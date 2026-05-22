#[derive(Debug, Clone, PartialEq)]
pub struct SpawnGroup {
    pub r#type: String,
    pub end: i32,
    pub begin: i32,
    pub spacing: i32,
    pub max: i32,
    pub unit_scaling: f32,
    pub shields: f32,
    pub shield_scaling: f32,
    pub unit_amount: i32,
    pub spawn: i32,
    pub payloads: Vec<String>,
    pub effect: Option<String>,
    pub items: Option<crate::mindustry::r#type::ItemStack>,
    pub team: Option<i32>,
}

impl SpawnGroup {
    pub const NEVER: i32 = i32::MAX;

    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            r#type: type_name.into(),
            end: Self::NEVER,
            begin: 0,
            spacing: 1,
            max: 40,
            unit_scaling: Self::NEVER as f32,
            shields: 0.0,
            shield_scaling: 0.0,
            unit_amount: 1,
            spawn: -1,
            payloads: Vec::new(),
            effect: None,
            items: None,
            team: None,
        }
    }

    pub fn can_spawn(&self, position: i32) -> bool {
        self.spawn == -1 || self.spawn == position
    }

    pub fn get_spawned(&self, wave: i32) -> i32 {
        let spacing = if self.spacing == 0 { 1 } else { self.spacing };
        if wave < self.begin || wave > self.end || (wave - self.begin) % spacing != 0 {
            return 0;
        }

        let stage = (wave - self.begin) / spacing;
        let scaling = if self.unit_scaling == 0.0 {
            0
        } else {
            (stage as f32 / self.unit_scaling) as i32
        };
        (self.unit_amount + scaling).min(self.max)
    }

    pub fn get_shield(&self, wave: i32) -> f32 {
        (self.shields + self.shield_scaling * (wave - self.begin) as f32).max(0.0)
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }
}

impl Default for SpawnGroup {
    fn default() -> Self {
        Self::new("dagger")
    }
}

#[cfg(test)]
mod tests {
    use super::SpawnGroup;

    #[test]
    fn get_spawned_respects_spacing_and_scaling() {
        let mut group = SpawnGroup::new("flare");
        group.begin = 0;
        group.end = 10;
        group.spacing = 1;
        group.unit_amount = 1;
        group.unit_scaling = 2.0;
        group.max = 3;

        assert_eq!(group.get_spawned(0), 1);
        assert_eq!(group.get_spawned(1), 1);
        assert_eq!(group.get_spawned(3), 2);
        assert_eq!(group.get_spawned(9), 3);
        assert_eq!(group.get_spawned(11), 0);
    }

    #[test]
    fn can_spawn_matches_java_spawn_filter() {
        let mut group = SpawnGroup::new("dagger");
        assert!(group.can_spawn(-100));
        assert!(group.can_spawn(42));

        group.spawn = 42;
        assert!(group.can_spawn(42));
        assert!(!group.can_spawn(41));
    }

    #[test]
    fn get_shield_scales_from_begin_and_clamps_to_zero() {
        let mut group = SpawnGroup::new("crawler");
        group.begin = 5;
        group.shields = 10.0;
        group.shield_scaling = 1.5;

        assert_eq!(group.get_shield(5), 10.0);
        assert_eq!(group.get_shield(7), 13.0);

        group.shields = 1.0;
        group.shield_scaling = -2.0;
        assert_eq!(group.get_shield(7), 0.0);
    }

    #[test]
    fn copy_clones_spawn_group_like_java_clone_wrapper() {
        let mut group = SpawnGroup::new("poly");
        group.begin = 2;
        group.end = 9;
        group.effect = Some("boss".into());

        assert_eq!(group.copy(), group);
    }
}
