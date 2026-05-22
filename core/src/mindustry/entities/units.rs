use crate::mindustry::io::{Point2, TypeValue};
use crate::mindustry::r#type::{StatusEffect, Weapon};
use crate::mindustry::world::block::Block;

#[derive(Debug, Clone, PartialEq)]
pub struct StatusEntry {
    pub effect: Option<StatusEffect>,
    pub time: f32,
    /// for interval damage
    pub damage_time: f32,

    /// all of these are for the dynamic effect only!
    pub damage_multiplier: f32,
    pub health_multiplier: f32,
    pub speed_multiplier: f32,
    pub reload_multiplier: f32,
    pub build_speed_multiplier: f32,
    pub drag_multiplier: f32,
    pub armor_override: f32,
}

impl StatusEntry {
    pub fn new(effect: StatusEffect, time: f32) -> Self {
        Self {
            effect: Some(effect),
            time,
            damage_time: 0.0,
            damage_multiplier: 1.0,
            health_multiplier: 1.0,
            speed_multiplier: 1.0,
            reload_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            drag_multiplier: 1.0,
            armor_override: -1.0,
        }
    }

    pub fn set(&mut self, effect: StatusEffect, time: f32) -> &mut Self {
        self.effect = Some(effect);
        self.time = time;
        self
    }
}

impl Default for StatusEntry {
    fn default() -> Self {
        Self {
            effect: None,
            time: 0.0,
            damage_time: 0.0,
            damage_multiplier: 1.0,
            health_multiplier: 1.0,
            speed_multiplier: 1.0,
            reload_multiplier: 1.0,
            build_speed_multiplier: 1.0,
            drag_multiplier: 1.0,
            armor_override: -1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponMount {
    /// weapon associated with this mount
    pub weapon: Weapon,
    /// reload in frames; 0 means ready to fire
    pub reload: f32,
    /// rotation relative to the unit this mount is on
    pub rotation: f32,
    /// weapon recoil
    pub recoil: f32,
    /// weapon barrel recoil
    pub recoils: Option<Vec<f32>>,
    /// destination rotation; do not modify!
    pub target_rotation: f32,
    /// current heat, 0 to 1
    pub heat: f32,
    /// lerps to 1 when shooting, 0 when not
    pub warmup: f32,
    /// is the weapon actively charging
    pub charging: bool,
    /// counts up to 1 when charging, 0 when not
    pub charge: f32,
    /// lerps to reload time
    pub smooth_reload: f32,
    /// aiming position in world coordinates
    pub aim_x: f32,
    pub aim_y: f32,
    /// whether to shoot right now
    pub shoot: bool,
    /// whether to allow any shooting effects
    pub allow_shoot_effects: bool,
    /// whether to rotate to face the target right now
    pub rotate: bool,
    /// extra state for alternating weapons
    pub side: bool,
    /// total bullets fired from this mount
    pub total_shots: i32,
    /// counter for which barrel bullets have been fired from; used for alternating patterns
    pub barrel_counter: i32,
    /// Last aim length of weapon. Only used for point lasers.
    pub last_length: f32,
    /// current bullet for continuous weapons
    pub bullet: Option<String>,
    /// sound loop for continuous weapons
    pub sound: Option<String>,
    /// current target; used for autonomous weapons and AI
    pub target: Option<String>,
    /// retarget counter
    pub retarget: f32,
}

impl WeaponMount {
    pub fn new(weapon: Weapon) -> Self {
        let rotation = weapon.base_rotation;
        Self {
            weapon,
            reload: 0.0,
            rotation,
            recoil: 0.0,
            recoils: None,
            target_rotation: rotation,
            heat: 0.0,
            warmup: 0.0,
            charging: false,
            charge: 0.0,
            smooth_reload: 0.0,
            aim_x: 0.0,
            aim_y: 0.0,
            shoot: false,
            allow_shoot_effects: true,
            rotate: false,
            side: false,
            total_shots: 0,
            barrel_counter: 0,
            last_length: 0.0,
            bullet: None,
            sound: None,
            target: None,
            retarget: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BuildPlan {
    /// Position and rotation of this plan.
    pub x: i32,
    pub y: i32,
    pub rotation: i32,
    /// Block being placed. If null, this is a breaking plan.
    pub block: Option<String>,
    /// Whether this is a break plan.
    pub breaking: bool,
    /// Config value, matching Java `BuildPlan.config` / `TypeIO.writeObject`.
    pub config: TypeValue,

    /// Last progress.
    pub progress: f32,
    /// Whether construction has started for this plan.
    pub initialized: bool,
    pub stuck: bool,
    pub cached_valid: bool,
    /// If true, this plan is in the world. If false, it is being rendered in a schematic.
    pub world_context: bool,

    /// Visual scale. Used only for rendering.
    pub anim_scale: f32,
}

impl BuildPlan {
    pub fn new_place(x: i32, y: i32, rotation: i32, block: impl Into<String>) -> Self {
        Self {
            x,
            y,
            rotation,
            block: Some(block.into()),
            breaking: false,
            config: TypeValue::Null,
            progress: 0.0,
            initialized: false,
            stuck: false,
            cached_valid: false,
            world_context: true,
            anim_scale: 0.0,
        }
    }

    pub fn new_place_block(x: i32, y: i32, rotation: i32, block: &Block) -> Self {
        Self::new_place(x, y, block.plan_rotation(rotation), block.name.clone())
    }

    pub fn new_config(
        x: i32,
        y: i32,
        rotation: i32,
        block: impl Into<String>,
        config: TypeValue,
    ) -> Self {
        Self {
            config,
            ..Self::new_place(x, y, rotation, block)
        }
    }

    pub fn new_config_block(
        x: i32,
        y: i32,
        rotation: i32,
        block: &Block,
        config: TypeValue,
    ) -> Self {
        Self {
            config,
            ..Self::new_place_block(x, y, rotation, block)
        }
    }

    pub fn new_string_config(
        x: i32,
        y: i32,
        rotation: i32,
        block: impl Into<String>,
        config: impl Into<String>,
    ) -> Self {
        Self::new_config(x, y, rotation, block, TypeValue::String(config.into()))
    }

    pub fn new_break(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            rotation: -1,
            block: None,
            breaking: true,
            config: TypeValue::Null,
            progress: 0.0,
            initialized: false,
            stuck: false,
            cached_valid: false,
            world_context: true,
            anim_scale: 0.0,
        }
    }

    pub fn same_pos(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn set_place(&mut self, x: i32, y: i32, rotation: i32, block: impl Into<String>) {
        self.x = x;
        self.y = y;
        self.rotation = rotation;
        self.block = Some(block.into());
        self.breaking = false;
    }

    pub fn set_place_block(&mut self, x: i32, y: i32, rotation: i32, block: &Block) {
        self.set_place(x, y, block.plan_rotation(rotation), block.name.clone());
    }

    pub fn set_break(&mut self) {
        self.rotation = -1;
        self.block = None;
        self.breaking = true;
    }

    pub fn point_config_value<F>(config: &TypeValue, mut transform: F) -> TypeValue
    where
        F: FnMut(Point2) -> Point2,
    {
        match config {
            TypeValue::Point2(point) => TypeValue::Point2(transform(*point)),
            TypeValue::Point2Array(points) => {
                TypeValue::Point2Array(points.iter().copied().map(transform).collect())
            }
            _ => config.clone(),
        }
    }

    pub fn point_config<F>(&mut self, transform: F)
    where
        F: FnMut(Point2) -> Point2,
    {
        self.config = Self::point_config_value(&self.config, transform);
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }
}

impl Default for BuildPlan {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            rotation: 0,
            block: None,
            breaking: false,
            config: TypeValue::Null,
            progress: 0.0,
            initialized: false,
            stuck: false,
            cached_valid: false,
            world_context: true,
            anim_scale: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mindustry::io::{Point2, TypeValue};
    use crate::mindustry::r#type::{StatusEffect, Weapon};
    use crate::mindustry::world::block::Block;

    use super::{BuildPlan, StatusEntry, WeaponMount};

    #[test]
    fn status_entry_set_attaches_effect_and_time() {
        let effect = StatusEffect::new(1, "burning");
        let mut entry = StatusEntry::default();

        entry.set(effect.clone(), 30.0);

        assert_eq!(entry.effect, Some(effect));
        assert_eq!(entry.time, 30.0);
        assert_eq!(entry.damage_multiplier, 1.0);
    }

    #[test]
    fn weapon_mount_uses_weapon_base_rotation() {
        let mut weapon = Weapon::new("duo");
        weapon.base_rotation = 45.0;

        let mount = WeaponMount::new(weapon.clone());

        assert_eq!(mount.weapon, weapon);
        assert_eq!(mount.rotation, 45.0);
        assert_eq!(mount.target_rotation, 45.0);
    }

    #[test]
    fn build_plan_supports_place_break_and_copy() {
        let mut plan = BuildPlan::new_place(3, 4, 1, "duo".to_string());
        assert_eq!(plan.block.as_deref(), Some("duo"));
        assert!(!plan.breaking);
        assert!(plan.same_pos(&BuildPlan::new_break(3, 4)));
        assert_eq!(plan.config, TypeValue::Null);

        let configured =
            BuildPlan::new_config(5, 6, 2, "router", TypeValue::Point2(Point2::new(1, 2)));
        assert_eq!(configured.config, TypeValue::Point2(Point2::new(1, 2)));

        let string_configured = BuildPlan::new_string_config(5, 6, 2, "router", "alpha");
        assert_eq!(string_configured.config, TypeValue::String("alpha".into()));

        plan.set_break();
        assert!(plan.breaking);
        assert_eq!(plan.rotation, -1);
        assert_eq!(plan.block, None);

        let copy = plan.copy();
        assert_eq!(copy, plan);
    }

    #[test]
    fn build_plan_point_config_transforms_point_values_without_losing_type() {
        let mut plan =
            BuildPlan::new_config(10, 20, 0, "router", TypeValue::Point2(Point2::new(1, 2)));

        plan.point_config(|point| Point2::new(point.x + 10, point.y - 1));

        assert_eq!(plan.config, TypeValue::Point2(Point2::new(11, 1)));

        plan.config = TypeValue::Point2Array(vec![Point2::new(0, 0), Point2::new(2, 3)]);
        plan.point_config(|point| Point2::new(point.x * 2, point.y * 3));

        assert_eq!(
            plan.config,
            TypeValue::Point2Array(vec![Point2::new(0, 0), Point2::new(4, 9)])
        );

        let string_config = TypeValue::String("unchanged".into());
        assert_eq!(
            BuildPlan::point_config_value(&string_config, |point| {
                Point2::new(point.x + 1, point.y + 1)
            }),
            string_config
        );
    }

    #[test]
    fn build_plan_block_helpers_apply_block_plan_rotation() {
        let mut block = Block::new(5, "sorter");

        let locked = BuildPlan::new_place_block(1, 2, 3, &block);
        assert_eq!(locked.block.as_deref(), Some("sorter"));
        assert_eq!(locked.rotation, 0);

        block.rotate = true;
        let rotating =
            BuildPlan::new_config_block(3, 4, 5, &block, TypeValue::String("cfg".into()));
        assert_eq!(rotating.rotation, 1);
        assert_eq!(rotating.config, TypeValue::String("cfg".into()));

        block.rotate = false;
        block.lock_rotation = false;
        let mut plan = BuildPlan::new_break(0, 0);
        plan.set_place_block(7, 8, -1, &block);
        assert_eq!((plan.x, plan.y, plan.rotation), (7, 8, 3));
        assert_eq!(plan.block.as_deref(), Some("sorter"));
        assert!(!plan.breaking);
    }
}
