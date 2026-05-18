use crate::mindustry::{
    ctype::ContentId,
    r#type::Item,
    world::{Block, BlockId, CacheLayer},
};

#[derive(Debug, Clone, PartialEq)]
pub struct FloorData {
    pub base: Block,
    pub overlay_floor: bool,
    pub edge: String,
    pub speed_multiplier: f32,
    pub drag_multiplier: f32,
    pub damage_taken: f32,
    pub drown_time: f32,
    pub walk_effect: String,
    pub walk_sound: String,
    pub walk_sound_volume: f32,
    pub walk_sound_pitch_min: f32,
    pub walk_sound_pitch_max: f32,
    pub drown_update_effect: String,
    pub status: String,
    pub status_duration: f32,
    pub liquid_drop: Option<ContentId>,
    pub liquid_multiplier: f32,
    pub is_liquid: bool,
    pub overlay_alpha: f32,
    pub supports_overlay: bool,
    pub shallow: bool,
    pub blend_group: BlockId,
    pub ore_default: bool,
    pub ore_scale: f32,
    pub ore_threshold: f32,
    pub wall: BlockId,
    pub decoration: BlockId,
    pub can_shadow: bool,
    pub force_draw_light: bool,
    pub needs_surface: bool,
    pub allow_core_placement: bool,
    pub wall_ore: bool,
    pub blend_id: i32,
    pub tiling_variants: i32,
    pub autotile: bool,
    pub autotile_mid_variants: i32,
    pub autotile_variants: i32,
    pub draw_edge_in: bool,
    pub draw_edge_out: bool,
}

impl FloorData {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        Self::with_variants(id, name, 3)
    }

    pub fn with_variants(id: BlockId, name: impl Into<String>, variants: i32) -> Self {
        let mut base = Block::new(id, name);
        base.replaceable = true;
        base.variants = variants;
        base.placeable_liquid = true;
        base.allow_rectangle_placement = true;
        base.instant_build = true;
        base.ignore_build_darkness = true;
        base.obstructs_light = false;
        base.place_effect = "rotateBlock".into();
        Self {
            blend_group: base.id,
            base,
            overlay_floor: false,
            edge: "stone".into(),
            speed_multiplier: 1.0,
            drag_multiplier: 1.0,
            damage_taken: 0.0,
            drown_time: 0.0,
            walk_effect: "none".into(),
            walk_sound: "none".into(),
            walk_sound_volume: 0.1,
            walk_sound_pitch_min: 0.8,
            walk_sound_pitch_max: 1.2,
            drown_update_effect: "bubble".into(),
            status: "none".into(),
            status_duration: 60.0,
            liquid_drop: None,
            liquid_multiplier: 1.0,
            is_liquid: false,
            overlay_alpha: 0.65,
            supports_overlay: false,
            shallow: false,
            ore_default: false,
            ore_scale: 24.0,
            ore_threshold: 0.828,
            wall: 0,
            decoration: 0,
            can_shadow: true,
            force_draw_light: false,
            needs_surface: true,
            allow_core_placement: false,
            wall_ore: false,
            blend_id: -1,
            tiling_variants: 0,
            autotile: false,
            autotile_mid_variants: 1,
            autotile_variants: 1,
            draw_edge_in: true,
            draw_edge_out: true,
        }
    }

    pub fn init_links(&mut self, wall: Option<BlockId>, decoration: Option<BlockId>) {
        self.blend_id = self.blend_group as i32;
        self.wall = wall.unwrap_or(0);
        self.decoration = decoration.unwrap_or(0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticWallData {
    pub base: Block,
    pub large_region: Option<String>,
    pub autotile: bool,
    pub autotile_mid_variants: i32,
}

impl StaticWallData {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        let mut base = Block::new(id, name);
        base.breakable = false;
        base.always_replace = false;
        base.unit_move_breakable = false;
        base.solid = true;
        base.variants = 2;
        base.cache_layer = CacheLayer::Walls;
        base.allow_rectangle_placement = true;
        base.place_effect = "rotateBlock".into();
        base.instant_build = true;
        base.ignore_build_darkness = true;
        base.placeable_liquid = true;
        Self {
            base,
            large_region: None,
            autotile: false,
            autotile_mid_variants: 1,
        }
    }

    pub fn can_replace(&self, other: &Block) -> bool {
        other.cache_layer == CacheLayer::Walls || self.base.always_replace || other.replaceable
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticTreeData {
    pub wall: StaticWallData,
}

impl StaticTreeData {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        let mut wall = StaticWallData::new(id, name);
        wall.base.variants = 0;
        Self { wall }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeBlockData {
    pub base: Block,
    pub shadow_offset: f32,
    pub layer: f32,
    pub shadow_layer: f32,
}

impl TreeBlockData {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        let mut base = Block::new(id, name);
        base.solid = true;
        base.clip_size = 90.0;
        base.custom_shadow = true;
        base.derive_layout_fields();
        Self {
            base,
            shadow_offset: -4.0,
            layer: 71.0,
            shadow_layer: 69.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TallBlockData {
    pub base: Block,
    pub shadow_offset: f32,
    pub layer: f32,
    pub shadow_layer: f32,
    pub rotation_rand: f32,
    pub shadow_alpha: f32,
}

impl TallBlockData {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        let mut base = Block::new(id, name);
        base.solid = true;
        base.clip_size = 90.0;
        base.custom_shadow = true;
        base.has_shadow = true;
        base.derive_layout_fields();
        Self {
            base,
            shadow_offset: -3.0,
            layer: 116.0,
            shadow_layer: 114.0,
            rotation_rand: 20.0,
            shadow_alpha: 0.6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropKind {
    Prop,
    Seaweed,
    SeaBush,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeaBushData {
    pub lobes_min: i32,
    pub lobes_max: i32,
    pub bot_angle: f32,
    pub origin: f32,
    pub scl_min: f32,
    pub scl_max: f32,
    pub mag_min: f32,
    pub mag_max: f32,
    pub time_range: f32,
    pub spread: f32,
}

impl Default for SeaBushData {
    fn default() -> Self {
        Self {
            lobes_min: 7,
            lobes_max: 7,
            bot_angle: 60.0,
            origin: 0.1,
            scl_min: 30.0,
            scl_max: 50.0,
            mag_min: 5.0,
            mag_max: 15.0,
            time_range: 40.0,
            spread: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropData {
    pub base: Block,
    pub kind: PropKind,
    pub layer: f32,
    pub sea_bush: Option<SeaBushData>,
}

impl PropData {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        Self::with_kind(id, name, PropKind::Prop)
    }

    pub fn with_kind(id: BlockId, name: impl Into<String>, kind: PropKind) -> Self {
        let mut base = Block::new(id, name);
        base.breakable = true;
        base.always_replace = true;
        base.instant_deconstruct = true;
        base.unit_move_breakable = true;
        base.break_effect = "breakProp".into();
        base.break_sound = "rockBreak".into();
        if matches!(kind, PropKind::Seaweed | PropKind::SeaBush) {
            base.obstructs_light = false;
        }
        if matches!(kind, PropKind::SeaBush) {
            base.variants = 0;
        }
        Self {
            base,
            kind,
            layer: 32.0,
            sea_bush: matches!(kind, PropKind::SeaBush).then(SeaBushData::default),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OreBlockData {
    pub floor: FloorData,
}

impl OreBlockData {
    pub fn new(id: BlockId, ore: &Item) -> Self {
        Self::with_name(id, format!("ore-{}", ore.base.mappable.name), ore)
    }

    pub fn with_name(id: BlockId, name: impl Into<String>, ore: &Item) -> Self {
        let mut floor = FloorData::new(id, name);
        floor.base.localized_name = Some(item_localized_name(ore));
        floor.base.item_drop = Some(ore.base.mappable.base.id);
        floor.base.variants = 3;
        floor.base.map_color_rgba = ore.color_rgba;
        floor.base.use_color = true;
        Self { floor }
    }

    pub fn setup(&mut self, ore: &Item) {
        self.floor.base.localized_name = Some(if self.floor.wall_ore {
            format!("{} wall ore", item_localized_name(ore))
        } else {
            item_localized_name(ore)
        });
        self.floor.base.item_drop = Some(ore.base.mappable.base.id);
        self.floor.base.map_color_rgba = ore.color_rgba;
    }

    pub fn init(&mut self, ore: Option<&Item>) -> Result<(), String> {
        match ore {
            Some(ore) => {
                self.setup(ore);
                Ok(())
            }
            None if self.floor.base.item_drop.is_some() => Ok(()),
            None => Err(format!("{} must have an item drop!", self.floor.base.name)),
        }
    }
}

fn item_localized_name(item: &Item) -> String {
    item.base
        .localized_name
        .clone()
        .unwrap_or_else(|| item.base.mappable.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::r#type::Item;

    #[test]
    fn floor_defaults_match_environment_constructor() {
        let floor = FloorData::new(1, "stone");
        assert!(!floor.overlay_floor);
        assert_eq!(floor.edge, "stone");
        assert_eq!(floor.speed_multiplier, 1.0);
        assert_eq!(floor.status, "none");
        assert!(floor.can_shadow);
        assert_eq!(floor.ore_scale, 24.0);
        assert_eq!(floor.ore_threshold, 0.828);
        assert_eq!(floor.wall, 0);
    }

    #[test]
    fn static_wall_defaults_match_upstream_constructor() {
        let wall = StaticWallData::new(2, "stone-wall");
        assert!(wall.base.solid);
        assert!(!wall.base.breakable);
        assert!(!wall.base.always_replace);
        assert!(!wall.base.unit_move_breakable);
        assert_eq!(wall.base.variants, 2);
        assert_eq!(wall.base.cache_layer, CacheLayer::Walls);
        assert!(wall.base.allow_rectangle_placement);
        assert_eq!(wall.base.place_effect, "rotateBlock");
        assert!(wall.base.instant_build);
        assert!(wall.base.ignore_build_darkness);
        assert!(wall.base.placeable_liquid);
        assert!(wall.large_region.is_none());
        assert!(!wall.autotile);
        assert_eq!(wall.autotile_mid_variants, 1);
    }

    #[test]
    fn static_wall_can_replace_matches_static_wall_or_block_default_subset() {
        let wall = StaticWallData::new(2, "stone-wall");

        let mut another_static_wall = Block::new(3, "dirt-wall");
        another_static_wall.cache_layer = CacheLayer::Walls;
        assert!(wall.can_replace(&another_static_wall));

        let mut replaceable_floor = Block::new(4, "sand");
        replaceable_floor.replaceable = true;
        assert!(wall.can_replace(&replaceable_floor));

        let solid_block = Block::new(5, "router");
        assert!(!wall.can_replace(&solid_block));
    }

    #[test]
    fn static_tree_tree_block_and_tall_block_defaults_match_upstream_subset() {
        let static_tree = StaticTreeData::new(3, "pine");
        assert_eq!(static_tree.wall.base.variants, 0);
        assert_eq!(static_tree.wall.base.cache_layer, CacheLayer::Walls);

        let tree = TreeBlockData::new(4, "white-tree");
        assert!(tree.base.solid);
        assert_eq!(tree.base.clip_size, 90.0);
        assert!(tree.base.custom_shadow);
        assert_eq!(tree.shadow_offset, -4.0);
        assert_eq!(tree.layer, 71.0);
        assert_eq!(tree.shadow_layer, 69.0);

        let tall = TallBlockData::new(5, "crystal-cluster");
        assert!(tall.base.solid);
        assert_eq!(tall.base.clip_size, 90.0);
        assert!(tall.base.custom_shadow);
        assert!(tall.base.has_shadow);
        assert_eq!(tall.shadow_offset, -3.0);
        assert_eq!(tall.rotation_rand, 20.0);
        assert_eq!(tall.shadow_alpha, 0.6);
    }

    #[test]
    fn prop_defaults_match_environment_constructor_subset() {
        let prop = PropData::new(3, "boulder");
        assert!(prop.base.breakable);
        assert!(prop.base.always_replace);
        assert!(prop.base.instant_deconstruct);
        assert!(prop.base.unit_move_breakable);
        assert_eq!(prop.base.break_effect, "breakProp");
        assert_eq!(prop.base.break_sound, "rockBreak");
        assert_eq!(prop.kind, PropKind::Prop);
        assert_eq!(prop.layer, 32.0);
        assert!(prop.sea_bush.is_none());

        let sea_bush = PropData::with_kind(4, "pur-bush", PropKind::SeaBush);
        assert_eq!(sea_bush.kind, PropKind::SeaBush);
        assert_eq!(sea_bush.base.variants, 0);
        assert!(!sea_bush.base.obstructs_light);
        assert!(sea_bush.sea_bush.is_some());
    }

    #[test]
    fn ore_block_setup_uses_item_drop_and_color() {
        let mut item = Item::new(7, "copper");
        item.color_rgba = 0xffaa00ff;
        let mut ore = OreBlockData::new(3, &item);
        assert_eq!(ore.floor.base.item_drop, Some(7));
        assert_eq!(ore.floor.base.map_color_rgba, 0xffaa00ff);

        ore.floor.wall_ore = true;
        ore.setup(&item);
        assert_eq!(
            ore.floor.base.localized_name.as_deref(),
            Some("copper wall ore")
        );
    }
}
