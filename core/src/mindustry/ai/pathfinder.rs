pub const IMPASSABLE: i32 = -1;
pub const FIELD_CORE: i32 = 0;
pub const MAX_FIELDS: i32 = 10;

pub const COST_GROUND: i32 = 0;
pub const COST_LEGS: i32 = 1;
pub const COST_NAVAL: i32 = 2;
pub const COST_NEOPLASM: i32 = 3;
pub const COST_NONE: i32 = 4;
pub const COST_HOVER: i32 = 5;
pub const MAX_COSTS: i32 = 8;

const HEALTH_SHIFT: u32 = 0;
const TEAM_SHIFT: u32 = 8;
const BOOL_SHIFT: u32 = 16;

pub const BIT_SOLID: u32 = 1 << (BOOL_SHIFT + 0);
pub const BIT_LIQUID: u32 = 1 << (BOOL_SHIFT + 1);
pub const BIT_LEG_SOLID: u32 = 1 << (BOOL_SHIFT + 2);
pub const BIT_NEAR_LIQUID: u32 = 1 << (BOOL_SHIFT + 3);
pub const BIT_NEAR_GROUND: u32 = 1 << (BOOL_SHIFT + 4);
pub const BIT_NEAR_SOLID: u32 = 1 << (BOOL_SHIFT + 5);
pub const BIT_NEAR_LEG_SOLID: u32 = 1 << (BOOL_SHIFT + 6);
pub const BIT_DEEP: u32 = 1 << (BOOL_SHIFT + 7);
pub const BIT_DAMAGES: u32 = 1 << (BOOL_SHIFT + 8);
pub const BIT_ALL_DEEP: u32 = 1 << (BOOL_SHIFT + 9);
pub const BIT_NEAR_DEEP: u32 = 1 << (BOOL_SHIFT + 10);
pub const BIT_TEAM_PASSABLE: u32 = 1 << (BOOL_SHIFT + 11);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PathTile {
    pub health: u8,
    pub team: u8,
    pub solid: bool,
    pub liquid: bool,
    pub leg_solid: bool,
    pub near_liquid: bool,
    pub near_ground: bool,
    pub near_solid: bool,
    pub near_leg_solid: bool,
    pub deep: bool,
    pub damages: bool,
    pub all_deep: bool,
    pub near_deep: bool,
    pub team_passable: bool,
}

impl PathTile {
    pub fn pack(self) -> i32 {
        let mut bits = ((self.health as u32) << HEALTH_SHIFT) | ((self.team as u32) << TEAM_SHIFT);
        set_bit(&mut bits, BIT_SOLID, self.solid);
        set_bit(&mut bits, BIT_LIQUID, self.liquid);
        set_bit(&mut bits, BIT_LEG_SOLID, self.leg_solid);
        set_bit(&mut bits, BIT_NEAR_LIQUID, self.near_liquid);
        set_bit(&mut bits, BIT_NEAR_GROUND, self.near_ground);
        set_bit(&mut bits, BIT_NEAR_SOLID, self.near_solid);
        set_bit(&mut bits, BIT_NEAR_LEG_SOLID, self.near_leg_solid);
        set_bit(&mut bits, BIT_DEEP, self.deep);
        set_bit(&mut bits, BIT_DAMAGES, self.damages);
        set_bit(&mut bits, BIT_ALL_DEEP, self.all_deep);
        set_bit(&mut bits, BIT_NEAR_DEEP, self.near_deep);
        set_bit(&mut bits, BIT_TEAM_PASSABLE, self.team_passable);
        bits as i32
    }

    pub fn unpack(bits: i32) -> Self {
        let bits = bits as u32;
        Self {
            health: ((bits >> HEALTH_SHIFT) & 0xff) as u8,
            team: ((bits >> TEAM_SHIFT) & 0xff) as u8,
            solid: bits & BIT_SOLID != 0,
            liquid: bits & BIT_LIQUID != 0,
            leg_solid: bits & BIT_LEG_SOLID != 0,
            near_liquid: bits & BIT_NEAR_LIQUID != 0,
            near_ground: bits & BIT_NEAR_GROUND != 0,
            near_solid: bits & BIT_NEAR_SOLID != 0,
            near_leg_solid: bits & BIT_NEAR_LEG_SOLID != 0,
            deep: bits & BIT_DEEP != 0,
            damages: bits & BIT_DAMAGES != 0,
            all_deep: bits & BIT_ALL_DEEP != 0,
            near_deep: bits & BIT_NEAR_DEEP != 0,
            team_passable: bits & BIT_TEAM_PASSABLE != 0,
        }
    }
}

pub fn path_cost(cost_type: i32, team: u8, tile: i32) -> i32 {
    match cost_type {
        COST_GROUND => ground_cost(team, tile),
        COST_LEGS => legs_cost(team, tile),
        COST_NAVAL => naval_cost(team, tile),
        COST_NEOPLASM => neoplasm_cost(team, tile),
        COST_NONE => none_cost(team, tile),
        COST_HOVER => hover_cost(team, tile),
        _ => none_cost(team, tile),
    }
}

pub fn ground_cost(team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    if tile.all_deep
        || (((tile.team == team && !tile.team_passable) || tile.team == 0) && tile.solid)
    {
        IMPASSABLE
    } else {
        1 + tile.health as i32 * 5
            + if tile.near_solid { 2 } else { 0 }
            + if tile.near_liquid { 6 } else { 0 }
            + if tile.deep { 6000 } else { 0 }
            + if tile.damages { 30 } else { 0 }
    }
}

pub fn legs_cost(_team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    if tile.leg_solid {
        IMPASSABLE
    } else {
        1 + if tile.deep { 6000 } else { 0 } + if tile.solid { 5 } else { 0 }
    }
}

pub fn naval_cost(team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    let base = if !tile.liquid || (tile.solid && (tile.team == team || tile.team == 0)) {
        7000
    } else {
        1
    };

    base + tile.health as i32 * 5
        + if tile.near_ground || tile.near_solid {
            14
        } else {
            0
        }
        + if tile.deep { 0 } else { 1 }
        + if tile.damages { 35 } else { 0 }
}

pub fn neoplasm_cost(_team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    if tile.deep || (tile.team == 0 && tile.solid) {
        IMPASSABLE
    } else {
        1 + tile.health as i32 * 3
            + if tile.near_solid { 2 } else { 0 }
            + if tile.near_liquid { 2 } else { 0 }
    }
}

pub fn none_cost(_team: u8, _tile: i32) -> i32 {
    1
}

pub fn hover_cost(team: u8, tile: i32) -> i32 {
    let tile = PathTile::unpack(tile);
    if ((tile.team == team && !tile.team_passable) || tile.team == 0) && tile.solid {
        IMPASSABLE
    } else {
        1 + tile.health as i32 * 5 + if tile.near_solid { 2 } else { 0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flowfield {
    pub refresh_rate: i32,
    pub has_complete: bool,
    pub dirty: bool,
    pub weights: Vec<i32>,
    pub searches: Vec<i16>,
    pub complete_weights: Vec<i32>,
    pub resolution: i32,
    pub width: i32,
    pub height: i32,
    pub targets: Vec<usize>,
    pub search: i16,
    pub initialized: bool,
}

impl Flowfield {
    pub fn new(world_width: i32, world_height: i32, resolution: i32) -> Self {
        let resolution = resolution.max(1);
        Self {
            refresh_rate: 0,
            has_complete: false,
            dirty: false,
            weights: Vec::new(),
            searches: Vec::new(),
            complete_weights: Vec::new(),
            resolution,
            width: div_ceil(world_width.max(0), resolution),
            height: div_ceil(world_height.max(0), resolution),
            targets: Vec::new(),
            search: 1,
            initialized: false,
        }
    }

    pub fn setup(&mut self) {
        let length = (self.width * self.height).max(0) as usize;
        self.weights = vec![IMPASSABLE; length];
        self.searches = vec![0; length];
        self.complete_weights = vec![0; length];
        self.initialized = true;
    }

    pub fn has_targets(&self) -> bool {
        !self.targets.is_empty()
    }

    pub fn has_complete_weights(&self) -> bool {
        self.has_complete && !self.complete_weights.is_empty()
    }

    pub fn needs_refresh(&self) -> bool {
        self.refresh_rate == 0
    }

    pub fn passable(&self, cost_type: i32, team: u8, tiles: &[i32], pos: usize) -> bool {
        let Some(tile) = tiles.get(pos).copied() else {
            return false;
        };
        let amount = path_cost(cost_type, team, tile);
        amount != IMPASSABLE && !(cost_type == COST_NAVAL && amount >= 6000)
    }
}

fn set_bit(bits: &mut u32, mask: u32, value: bool) {
    if value {
        *bits |= mask;
    }
}

fn div_ceil(value: i32, divisor: i32) -> i32 {
    if value <= 0 {
        0
    } else {
        (value + divisor - 1) / divisor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile() -> PathTile {
        PathTile {
            health: 3,
            team: 2,
            solid: false,
            liquid: false,
            leg_solid: false,
            near_liquid: false,
            near_ground: false,
            near_solid: false,
            near_leg_solid: false,
            deep: false,
            damages: false,
            all_deep: false,
            near_deep: false,
            team_passable: false,
        }
    }

    #[test]
    fn path_tile_pack_roundtrips_health_team_and_flags() {
        let mut source = tile();
        source.solid = true;
        source.liquid = true;
        source.near_solid = true;
        source.damages = true;
        source.team_passable = true;

        let packed = source.pack();

        assert_eq!(PathTile::unpack(packed), source);
        assert_ne!(packed as u32 & BIT_SOLID, 0);
        assert_ne!(packed as u32 & BIT_TEAM_PASSABLE, 0);
    }

    #[test]
    fn ground_hover_and_leg_costs_follow_java_formulas() {
        let mut ground = tile();
        ground.health = 2;
        ground.near_solid = true;
        ground.near_liquid = true;
        ground.damages = true;
        assert_eq!(ground_cost(1, ground.pack()), 49);

        ground.solid = true;
        ground.team = 0;
        assert_eq!(ground_cost(1, ground.pack()), IMPASSABLE);
        assert_eq!(hover_cost(1, ground.pack()), IMPASSABLE);

        let mut legs = tile();
        legs.deep = true;
        legs.solid = true;
        assert_eq!(legs_cost(1, legs.pack()), 6006);
        legs.leg_solid = true;
        assert_eq!(legs_cost(1, legs.pack()), IMPASSABLE);
    }

    #[test]
    fn naval_neoplasm_and_none_costs_follow_java_formulas() {
        let mut naval = tile();
        naval.liquid = true;
        naval.deep = true;
        naval.health = 1;
        naval.near_ground = true;
        naval.damages = true;
        assert_eq!(naval_cost(1, naval.pack()), 55);

        naval.liquid = false;
        assert_eq!(naval_cost(1, naval.pack()), 7054);

        let mut neo = tile();
        neo.health = 2;
        neo.near_liquid = true;
        neo.near_solid = true;
        assert_eq!(neoplasm_cost(1, neo.pack()), 11);
        neo.team = 0;
        neo.solid = true;
        assert_eq!(neoplasm_cost(1, neo.pack()), IMPASSABLE);

        assert_eq!(none_cost(1, neo.pack()), 1);
    }

    #[test]
    fn flowfield_setup_and_passable_match_java_shape() {
        let mut field = Flowfield::new(10, 9, 2);
        assert_eq!((field.width, field.height), (5, 5));
        assert!(!field.initialized);

        field.setup();
        assert!(field.initialized);
        assert_eq!(field.weights.len(), 25);
        assert!(field.needs_refresh());
        assert!(!field.has_targets());

        let mut water = tile();
        water.liquid = false;
        let tiles = vec![water.pack()];
        assert!(!field.passable(COST_NAVAL, 1, &tiles, 0));

        water.liquid = true;
        water.deep = true;
        let tiles = vec![water.pack()];
        assert!(field.passable(COST_NAVAL, 1, &tiles, 0));
    }
}
