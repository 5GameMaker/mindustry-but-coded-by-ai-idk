use crate::mindustry::{
    ctype::ContentId,
    io::TypeValue,
    vars::TILE_SIZE,
    world::{
        build::BuildBounds,
        meta::{BlockFlag, BlockGroup, BuildVisibility, Env},
    },
};

pub type BlockId = ContentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CacheLayer {
    #[default]
    None,
    Water,
    Mud,
    Tar,
    Slag,
    Arkycite,
    Cryofluid,
    Space,
    Normal,
    Walls,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub id: BlockId,
    pub name: String,
    pub localized_name: Option<String>,

    pub has_items: bool,
    pub has_liquids: bool,
    pub has_power: bool,
    pub update: bool,
    pub destructible: bool,
    pub save_data: bool,
    pub sync: bool,
    pub configurable: bool,
    pub editor_configurable: bool,
    pub save_config: bool,
    pub copy_config: bool,
    pub clear_on_double_tap: bool,
    pub logic_configurable: bool,
    pub ignore_resize_config: bool,
    pub commandable: bool,
    pub allow_config_inventory: bool,
    pub selection_rows: i32,
    pub selection_columns: i32,
    pub consumes_tap: bool,
    pub last_config: Option<TypeValue>,

    pub solid: bool,
    pub solidifies: bool,
    pub fills_tile: bool,
    pub force_dark: bool,
    pub breakable: bool,
    pub always_replace: bool,
    pub replaceable: bool,
    pub instant_deconstruct: bool,
    pub unit_move_breakable: bool,
    pub rebuildable: bool,
    pub privileged: bool,
    pub no_update_disabled: bool,
    pub targetable: bool,
    pub health: i32,

    pub rotate: bool,
    pub rotate_draw: bool,
    pub rotate_draw_editor: bool,
    pub visual_rotation_offset: f32,
    pub lock_rotation: bool,
    pub ignore_line_rotation: bool,
    pub invert_flip: bool,
    pub quick_rotate: bool,

    pub size: i32,
    pub offset: f32,
    pub size_offset: i32,
    pub clip_size: f32,
    pub light_clip_size: f32,

    pub item_capacity: i32,
    pub liquid_capacity: f32,
    pub consumes_power: bool,
    pub outputs_power: bool,
    pub connected_power: bool,
    pub conductive_power: bool,

    pub cache_layer: CacheLayer,
    pub build_visibility: BuildVisibility,
    pub group: BlockGroup,
    pub flags: Vec<BlockFlag>,
    pub priority: i32,
    pub variants: i32,
    pub placeable_on: bool,
    pub placeable_liquid: bool,
    pub allow_rectangle_placement: bool,
    pub swap_diagonal_placement: bool,
    pub instant_build: bool,
    pub ignore_build_darkness: bool,
    pub obstructs_light: bool,
    pub custom_shadow: bool,
    pub has_shadow: bool,
    pub place_effect: String,
    pub break_effect: String,
    pub break_sound: String,
    pub albedo: f32,
    pub emit_light: bool,
    pub light_radius: f32,
    pub light_color_rgba: u32,
    pub can_overdrive: bool,
    pub suppressable: bool,

    pub env_required: u32,
    pub env_enabled: u32,
    pub env_disabled: u32,

    pub item_drop: Option<ContentId>,
    pub map_color_rgba: u32,
    pub use_color: bool,
    pub force_team: Option<i32>,
}

impl Block {
    pub fn new(id: BlockId, name: impl Into<String>) -> Self {
        let mut block = Self {
            id,
            name: name.into(),
            localized_name: None,
            has_items: false,
            has_liquids: false,
            has_power: false,
            update: false,
            destructible: false,
            save_data: false,
            sync: false,
            configurable: false,
            editor_configurable: false,
            save_config: false,
            copy_config: true,
            clear_on_double_tap: false,
            logic_configurable: false,
            ignore_resize_config: false,
            commandable: false,
            allow_config_inventory: true,
            selection_rows: 5,
            selection_columns: 4,
            consumes_tap: false,
            last_config: None,
            solid: false,
            solidifies: false,
            fills_tile: true,
            force_dark: false,
            breakable: true,
            always_replace: false,
            replaceable: true,
            instant_deconstruct: false,
            unit_move_breakable: true,
            rebuildable: true,
            privileged: false,
            no_update_disabled: false,
            targetable: true,
            health: 40,
            rotate: false,
            rotate_draw: true,
            rotate_draw_editor: true,
            visual_rotation_offset: 0.0,
            lock_rotation: true,
            ignore_line_rotation: false,
            invert_flip: false,
            quick_rotate: true,
            size: 1,
            offset: 0.0,
            size_offset: 0,
            clip_size: -1.0,
            light_clip_size: 0.0,
            item_capacity: 10,
            liquid_capacity: 10.0,
            consumes_power: true,
            outputs_power: false,
            connected_power: true,
            conductive_power: false,
            cache_layer: CacheLayer::None,
            build_visibility: BuildVisibility::Shown,
            group: BlockGroup::None,
            flags: Vec::new(),
            priority: 0,
            variants: 0,
            placeable_on: true,
            placeable_liquid: false,
            allow_rectangle_placement: false,
            swap_diagonal_placement: false,
            instant_build: false,
            ignore_build_darkness: false,
            obstructs_light: true,
            custom_shadow: false,
            has_shadow: false,
            place_effect: "none".into(),
            break_effect: "breakBlock".into(),
            break_sound: "bang".into(),
            albedo: 0.0,
            emit_light: false,
            light_radius: 0.0,
            light_color_rgba: 0x00000000,
            can_overdrive: true,
            suppressable: false,
            env_required: Env::NONE,
            env_enabled: Env::ANY,
            env_disabled: Env::NONE,
            item_drop: None,
            map_color_rgba: 0x00000000,
            use_color: false,
            force_team: None,
        };
        block.derive_layout_fields();
        block
    }

    pub fn derive_layout_fields(&mut self) {
        self.offset = ((self.size + 1) % 2) as f32 * TILE_SIZE as f32 / 2.0;
        self.size_offset = -((self.size - 1) / 2);
        self.clip_size = self.clip_size.max(self.size as f32 * TILE_SIZE as f32);
        self.light_clip_size = self.light_clip_size.max(self.clip_size);
    }

    pub const fn has_building(&self) -> bool {
        self.destructible || self.update
    }

    pub const fn is_multiblock(&self) -> bool {
        self.size > 1
    }

    pub const fn synthetic(&self) -> bool {
        self.update || self.destructible
    }

    pub const fn is_static(&self) -> bool {
        matches!(self.cache_layer, CacheLayer::Walls)
    }

    pub const fn supports_env(&self, env: u32) -> bool {
        (env & self.env_required) == self.env_required
            && (env & self.env_disabled) == 0
            && (env & self.env_enabled) != 0
    }

    pub fn display_name(&self) -> &str {
        self.localized_name.as_deref().unwrap_or(&self.name)
    }

    pub fn can_replace(&self, other: &Block) -> bool {
        if other.always_replace {
            return true;
        }
        if other.privileged || !other.replaceable {
            return false;
        }
        let same_block = self.id == other.id;
        let same_group = self.group != BlockGroup::None && self.group == other.group;
        let size_matches =
            self.size == other.size || (self.size >= other.size && self.group.any_replace());

        (same_block || same_group)
            && size_matches
            && (!same_block || (self.rotate && self.quick_rotate))
    }

    pub const fn can_be_built(&self) -> bool {
        !matches!(
            self.build_visibility,
            BuildVisibility::Hidden | BuildVisibility::DebugOnly
        )
    }

    pub fn is_darkened(&self) -> bool {
        self.solid && ((!self.synthetic() && self.fills_tile) || self.force_dark)
    }

    /// Java `Block.minimapColor(Tile)` default: return 0 so callers fall back
    /// to `mapColor` unless a concrete content definition overrides it.
    pub const fn minimap_color_rgba(&self) -> u32 {
        0
    }

    /// Java `Block.getColor(Tile)` default fallback encoded as packed RGBA.
    pub fn color_rgba(&self) -> u32 {
        let minimap = self.minimap_color_rgba();
        if minimap == 0 {
            self.map_color_rgba
        } else {
            minimap
        }
    }

    pub fn update_clip_radius(&mut self, radius_in_world_units: f32) {
        self.clip_size = self
            .clip_size
            .max(self.size as f32 * TILE_SIZE as f32 + radius_in_world_units * 2.0);
        self.light_clip_size = self.light_clip_size.max(self.clip_size);
    }

    pub fn bounds(&self, x: i32, y: i32) -> BuildBounds {
        let size = self.size.max(1) as f32 * TILE_SIZE as f32;
        BuildBounds {
            x: x as f32 * TILE_SIZE as f32 + self.offset - size / 2.0,
            y: y as f32 * TILE_SIZE as f32 + self.offset - size / 2.0,
            width: size,
            height: size,
        }
    }

    pub fn taken_positions(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        crate::mindustry::world::build::footprint_tiles(x, y, self.size)
    }

    pub fn plan_rotation(&self, rotation: i32) -> i32 {
        if !self.rotate && self.lock_rotation {
            0
        } else {
            rotation.rem_euclid(4)
        }
    }

    pub fn set_last_config(&mut self, config: TypeValue) {
        self.last_config = match config {
            TypeValue::Null => None,
            config => Some(config),
        };
    }

    pub fn next_config(&self) -> Option<TypeValue> {
        if self.save_config {
            self.last_config.clone()
        } else {
            None
        }
    }

    pub fn config_clearable(&self) -> bool {
        self.configurable && self.clear_on_double_tap
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new(0, "air")
    }
}

#[cfg(test)]
mod tests {
    use crate::mindustry::io::TypeValue;

    use super::{Block, CacheLayer};

    #[test]
    fn block_layout_fields_match_upstream_formula() {
        let mut one = Block::new(1, "one");
        one.size = 1;
        one.clip_size = -1.0;
        one.derive_layout_fields();
        assert_eq!(one.offset, 0.0);
        assert_eq!(one.size_offset, 0);
        assert_eq!(one.clip_size, 8.0);

        let mut two = Block::new(2, "two");
        two.size = 2;
        two.clip_size = -1.0;
        two.derive_layout_fields();
        assert_eq!(two.offset, 4.0);
        assert_eq!(two.size_offset, 0);
        assert_eq!(two.clip_size, 16.0);

        let mut three = Block::new(3, "three");
        three.size = 3;
        three.clip_size = -1.0;
        three.derive_layout_fields();
        assert_eq!(three.offset, 0.0);
        assert_eq!(three.size_offset, -1);
        assert!(three.is_multiblock());
    }

    #[test]
    fn block_flags_follow_java_helpers() {
        let mut block = Block::new(1, "wall");
        assert!(!block.has_building());
        block.destructible = true;
        assert!(block.has_building());
        assert!(block.synthetic());
        block.cache_layer = CacheLayer::Walls;
        assert!(block.is_static());
    }

    #[test]
    fn block_config_metadata_matches_upstream_defaults_and_helpers() {
        let mut block = Block::new(2, "router");

        assert!(!block.configurable);
        assert!(!block.editor_configurable);
        assert!(!block.save_config);
        assert!(block.copy_config);
        assert!(!block.clear_on_double_tap);
        assert!(!block.logic_configurable);
        assert!(!block.ignore_resize_config);
        assert!(!block.commandable);
        assert!(block.allow_config_inventory);
        assert_eq!((block.selection_rows, block.selection_columns), (5, 4));
        assert!(!block.consumes_tap);
        assert!(block.consumes_power);
        assert!(!block.outputs_power);
        assert!(block.connected_power);
        assert_eq!(block.next_config(), None);
        assert!(!block.config_clearable());

        block.configurable = true;
        block.clear_on_double_tap = true;
        assert!(block.config_clearable());

        block.set_last_config(TypeValue::String("cfg".into()));
        assert_eq!(block.next_config(), None);
        block.save_config = true;
        assert_eq!(block.next_config(), Some(TypeValue::String("cfg".into())));

        block.set_last_config(TypeValue::Null);
        assert_eq!(block.next_config(), None);
    }

    #[test]
    fn block_rotation_metadata_matches_plan_rotation_rules() {
        let mut block = Block::new(3, "sorter");

        assert!(!block.rotate);
        assert!(block.rotate_draw);
        assert!(block.rotate_draw_editor);
        assert!(block.lock_rotation);
        assert!(!block.ignore_line_rotation);
        assert!(!block.invert_flip);
        assert_eq!(block.plan_rotation(3), 0);

        block.rotate = true;
        assert_eq!(block.plan_rotation(5), 1);

        block.rotate = false;
        block.lock_rotation = false;
        assert_eq!(block.plan_rotation(-1), 3);
    }

    #[test]
    fn block_runtime_replacement_and_geometry_helpers_match_java_base() {
        let mut conveyor = Block::new(10, "conveyor");
        conveyor.group = crate::mindustry::world::meta::BlockGroup::Transportation;
        conveyor.rotate = true;

        let mut junction = Block::new(11, "junction");
        junction.group = crate::mindustry::world::meta::BlockGroup::Transportation;
        assert!(conveyor.can_replace(&junction));

        let same = conveyor.clone();
        assert!(conveyor.can_replace(&same));
        conveyor.rotate = false;
        assert!(!conveyor.can_replace(&same));

        let mut core = Block::new(12, "core");
        core.replaceable = false;
        assert!(!conveyor.can_replace(&core));
        core.always_replace = true;
        assert!(conveyor.can_replace(&core));

        let mut large = Block::new(13, "large");
        large.size = 2;
        large.derive_layout_fields();
        assert_eq!(
            large.taken_positions(4, 5),
            vec![(4, 5), (4, 6), (5, 5), (5, 6)]
        );
        assert_eq!(
            large.bounds(4, 5),
            crate::mindustry::world::BuildBounds {
                x: 28.0,
                y: 36.0,
                width: 16.0,
                height: 16.0,
            }
        );
        large.update_clip_radius(20.0);
        assert_eq!(large.clip_size, 56.0);
        assert_eq!(large.light_clip_size, 56.0);
    }

    #[test]
    fn block_runtime_defaults_expose_building_control_flags() {
        let block = Block::new(20, "base");
        assert!(block.replaceable);
        assert!(block.rebuildable);
        assert!(!block.privileged);
        assert!(!block.no_update_disabled);
        assert_eq!(block.visual_rotation_offset, 0.0);
        assert!(block.quick_rotate);
        assert!(!block.solidifies);
        assert!(!block.swap_diagonal_placement);
        assert!(block.can_overdrive);
        assert!(!block.suppressable);
        assert!(block.force_team.is_none());
    }

    #[test]
    fn block_color_helpers_match_java_minimap_and_mapcolor_fallback() {
        let mut block = Block::new(21, "colored");
        block.map_color_rgba = 0x11223344;

        assert_eq!(block.minimap_color_rgba(), 0);
        assert_eq!(block.color_rgba(), 0x11223344);
    }
}
