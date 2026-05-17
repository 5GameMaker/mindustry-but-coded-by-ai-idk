use crate::mindustry::{
    ctype::ContentId,
    vars::TILE_SIZE,
    world::meta::{BlockFlag, BlockGroup, BuildVisibility, Env},
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

    pub solid: bool,
    pub fills_tile: bool,
    pub force_dark: bool,
    pub breakable: bool,
    pub always_replace: bool,
    pub replaceable: bool,
    pub instant_deconstruct: bool,
    pub unit_move_breakable: bool,
    pub targetable: bool,
    pub health: i32,

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

    pub env_required: u32,
    pub env_enabled: u32,
    pub env_disabled: u32,

    pub item_drop: Option<ContentId>,
    pub map_color_rgba: u32,
    pub use_color: bool,
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
            solid: false,
            fills_tile: true,
            force_dark: false,
            breakable: true,
            always_replace: false,
            replaceable: false,
            instant_deconstruct: false,
            unit_move_breakable: true,
            targetable: true,
            health: 40,
            size: 1,
            offset: 0.0,
            size_offset: 0,
            clip_size: -1.0,
            light_clip_size: 0.0,
            item_capacity: 10,
            liquid_capacity: 10.0,
            consumes_power: false,
            outputs_power: false,
            connected_power: false,
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
            env_required: Env::NONE,
            env_enabled: Env::ANY,
            env_disabled: Env::NONE,
            item_drop: None,
            map_color_rgba: 0x00000000,
            use_color: false,
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
}

impl Default for Block {
    fn default() -> Self {
        Self::new(0, "air")
    }
}

#[cfg(test)]
mod tests {
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
}
