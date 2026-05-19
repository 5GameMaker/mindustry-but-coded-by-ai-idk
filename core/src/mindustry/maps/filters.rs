#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapBlockKind {
    Air,
    Floor,
    Wall,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapBlock {
    pub name: &'static str,
    pub kind: MapBlockKind,
    pub synthetic: bool,
    pub breakable: bool,
    pub has_surface: bool,
    pub needs_surface: bool,
}

impl MapBlock {
    pub const AIR: Self = Self::new("air", MapBlockKind::Air);
    pub const STONE: Self = Self::floor("stone", true);
    pub const STONE_WALL: Self = Self::wall("stone-wall", false, false);
    pub const SAND: Self = Self::floor("sand", true);
    pub const SAND_WATER: Self = Self::floor("sand-water", true);
    pub const ORE_COPPER: Self = Self::overlay("ore-copper", true);
    pub const SPAWN: Self = Self::overlay("spawn", true);

    pub const fn new(name: &'static str, kind: MapBlockKind) -> Self {
        Self {
            name,
            kind,
            synthetic: false,
            breakable: true,
            has_surface: false,
            needs_surface: false,
        }
    }

    pub const fn floor(name: &'static str, has_surface: bool) -> Self {
        Self {
            name,
            kind: MapBlockKind::Floor,
            synthetic: false,
            breakable: true,
            has_surface,
            needs_surface: false,
        }
    }

    pub const fn wall(name: &'static str, synthetic: bool, breakable: bool) -> Self {
        Self {
            name,
            kind: MapBlockKind::Wall,
            synthetic,
            breakable,
            has_surface: false,
            needs_surface: false,
        }
    }

    pub const fn overlay(name: &'static str, needs_surface: bool) -> Self {
        Self {
            name,
            kind: MapBlockKind::Overlay,
            synthetic: false,
            breakable: true,
            has_surface: false,
            needs_surface,
        }
    }

    pub const fn is_air(self) -> bool {
        matches!(self.kind, MapBlockKind::Air)
    }

    pub const fn is_floor(self) -> bool {
        matches!(self.kind, MapBlockKind::Floor)
    }

    pub const fn is_overlay(self) -> bool {
        matches!(self.kind, MapBlockKind::Overlay)
    }

    pub const fn is_solid(self) -> bool {
        matches!(self.kind, MapBlockKind::Wall)
    }

    pub const fn is_static(self) -> bool {
        self.is_solid() && !self.breakable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerateInput {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub floor: MapBlock,
    pub block: MapBlock,
    pub overlay: MapBlock,
    pub packed_data: i64,
}

impl GenerateInput {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            floor: MapBlock::STONE,
            block: MapBlock::AIR,
            overlay: MapBlock::AIR,
            packed_data: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedTile {
    pub block: i16,
    pub floor: i16,
    pub overlay: i16,
}

impl PackedTile {
    pub fn pack(block: i16, floor: i16, overlay: i16) -> i64 {
        ((block as u16 as i64) << 32) | ((floor as u16 as i64) << 16) | overlay as u16 as i64
    }

    pub fn unpack(value: i64) -> Self {
        Self {
            block: ((value >> 32) & 0xffff) as i16,
            floor: ((value >> 16) & 0xffff) as i16,
            overlay: (value & 0xffff) as i16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapTile {
    pub floor: MapBlock,
    pub block: MapBlock,
    pub overlay: MapBlock,
    pub packed_data: i64,
}

impl MapTile {
    pub const fn new(
        floor: MapBlock,
        block: MapBlock,
        overlay: MapBlock,
        packed_data: i64,
    ) -> Self {
        Self {
            floor,
            block,
            overlay,
            packed_data,
        }
    }

    pub fn from_input(input: &GenerateInput) -> Self {
        Self {
            floor: input.floor,
            block: input.block,
            overlay: input.overlay,
            packed_data: input.packed_data,
        }
    }

    pub fn overlay_id(self) -> i16 {
        PackedTile::unpack(self.packed_data).overlay
    }
}

pub fn filter_simple_name(class_name: &str) -> String {
    class_name.trim_end_matches("Filter").to_ascii_lowercase()
}

pub fn noise_sample_x(x: i32) -> f32 {
    x as f32 + 10.0
}

pub fn noise_sample_y(y: i32, x: i32, tilt: f32) -> f32 {
    y as f32 + x as f32 * tilt + 10.0
}

pub fn deterministic_chance(x: i32, y: i32, seed: i32) -> f32 {
    let mut state = ((x as u64) << 32) ^ ((y.wrapping_add(seed)) as u32 as u64);
    state ^= state >> 33;
    state = state.wrapping_mul(0xff51afd7ed558ccd);
    state ^= state >> 33;
    state = state.wrapping_mul(0xc4ceb9fe1a85ec53);
    state ^= state >> 33;
    ((state >> 40) as f32) / ((1u64 << 24) as f32)
}

pub fn clear_filter_apply(
    input: &mut GenerateInput,
    target: MapBlock,
    replace: MapBlock,
    ignore: MapBlock,
) {
    if !ignore.is_air()
        && (input.block == ignore || input.floor == ignore || input.overlay == ignore)
    {
        return;
    }

    if input.block == target
        || input.floor == target
        || (target.is_overlay() && input.overlay == target)
    {
        if replace.is_air() {
            if input.overlay == target {
                input.overlay = MapBlock::AIR;
            } else {
                input.block = MapBlock::AIR;
            }
        } else if replace.is_overlay() {
            input.overlay = replace;
        } else if replace.is_floor() {
            input.floor = replace;
        } else {
            input.block = replace;
        }
    }
}

pub fn noise_filter_apply(
    input: &mut GenerateInput,
    noise: f32,
    threshold: f32,
    target: MapBlock,
    floor: MapBlock,
    block: MapBlock,
) {
    if noise > threshold && (target.is_air() || input.floor == target || input.block == target) {
        if !floor.is_air() {
            input.floor = floor;
        }
        if !block.is_air() && !input.block.is_air() && !input.block.breakable {
            input.block = block;
        }
    }
}

pub fn terrain_filter_noise(
    noise: f32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    circle_scl: f32,
) -> f32 {
    noise + distance(x as f32 / width as f32, y as f32 / height as f32, 0.5, 0.5) * circle_scl
}

pub fn terrain_filter_apply(
    input: &mut GenerateInput,
    terrain_noise: f32,
    threshold: f32,
    floor: MapBlock,
    block: MapBlock,
) {
    if !floor.is_air() {
        input.floor = floor;
    }
    if terrain_noise >= threshold {
        input.block = block;
    }
}

pub fn ore_filter_apply(
    input: &mut GenerateInput,
    noise: f32,
    threshold: f32,
    ore: MapBlock,
    target: MapBlock,
) {
    if noise > threshold
        && input.overlay != MapBlock::SPAWN
        && (target.is_air() || input.floor == target || input.overlay == target)
        && input.floor.has_surface
    {
        input.overlay = ore;
    }
}

pub fn scatter_filter_apply(
    input: &mut GenerateInput,
    random: f32,
    chance: f32,
    floor_onto: MapBlock,
    floor: MapBlock,
    block: MapBlock,
) {
    let floor_matches = input.floor == floor_onto || floor_onto.is_air();
    if !block.is_air() && floor_matches && input.block.is_air() && random <= chance {
        if block.is_overlay() {
            input.overlay = block;
        } else {
            input.block = block;
        }
    }

    if !floor.is_air() && floor_matches && random <= chance {
        input.floor = floor;
    }
}

pub fn median_filter_pick(values: &mut [i16], percentile: f32) -> Option<i16> {
    if values.is_empty() {
        return None;
    }
    values.sort();
    let index = ((values.len() as f32 * percentile) as usize).min(values.len() - 1);
    Some(values[index])
}

pub fn circle_offsets(radius: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for x in -radius..=radius {
        for y in -radius..=radius {
            if x * x + y * y <= radius * radius {
                out.push((x, y));
            }
        }
    }
    out
}

pub fn blend_filter_should_skip(
    input: &GenerateInput,
    block: MapBlock,
    floor: MapBlock,
    ignore: MapBlock,
) -> bool {
    input.floor == block
        || block.is_air()
        || input.floor == ignore
        || (!floor.is_floor() && (input.block == block || input.block == ignore))
}

pub fn blend_filter_apply(input: &mut GenerateInput, found: bool, floor: MapBlock) {
    if found {
        if floor.is_floor() {
            input.floor = floor;
        } else {
            input.block = floor;
        }
    }
}

pub fn river_noise_filter_apply(
    input: &mut GenerateInput,
    noise: f32,
    threshold: f32,
    threshold2: f32,
    floor: MapBlock,
    floor2: MapBlock,
    block: MapBlock,
    target: MapBlock,
) {
    if noise >= threshold && (target.is_air() || input.floor == target || input.block == target) {
        if !floor.is_air() {
            input.floor = floor;
        }

        if input.block.is_solid() && !block.is_air() && !input.block.is_air() {
            input.block = block;
        }

        if noise >= threshold2 && !floor2.is_air() {
            input.floor = floor2;
        }
    }
}

pub fn clamped_tile_coord(x: f32, y: f32, width: i32, height: i32) -> (i32, i32) {
    (
        clamp_i32(x as i32, 0, (width - 1).max(0)),
        clamp_i32(y as i32, 0, (height - 1).max(0)),
    )
}

pub fn distort_filter_source_coord(
    input: &GenerateInput,
    noise_x: f32,
    noise_y: f32,
    mag: f32,
) -> (i32, i32) {
    clamped_tile_coord(
        input.x as f32 + noise_x - mag / 2.0,
        input.y as f32 + noise_y - mag / 2.0,
        input.width,
        input.height,
    )
}

pub fn distort_filter_apply(input: &mut GenerateInput, source: MapTile) {
    input.floor = source.floor;
    if !source.block.synthetic && !input.block.synthetic {
        input.block = source.block;
    }
    input.overlay = source.overlay;
}

pub fn mirror_filter_source_coord(
    input: &GenerateInput,
    angle: i32,
    rotate: bool,
) -> Option<(i32, i32)> {
    let (mut v1x, mut v1y) = vector_from_degrees(angle as f32 - 90.0, 1.0);
    let mut v2x = -v1x;
    let mut v2y = -v1y;
    let cx = input.width as f32 / 2.0 - 0.5;
    let cy = input.height as f32 / 2.0 - 0.5;

    v1x += cx;
    v1y += cy;
    v2x += cx;
    v2y += cy;

    let mut px = input.x as f32;
    let mut py = input.y as f32;

    if mirror_filter_left(v1x, v1y, v2x, v2y, px, py) {
        return None;
    }

    if (input.width != input.height && angle % 90 != 0) || rotate {
        px = input.width as f32 - px - 1.0;
        py = input.height as f32 - py - 1.0;
    } else {
        let dx = v2x - v1x;
        let dy = v2y - v1y;
        let denom = dx * dx + dy * dy;

        if denom != 0.0 {
            let a = (dx * dx - dy * dy) / denom;
            let b = 2.0 * dx * dy / denom;
            let relx = px - v1x;
            let rely = py - v1y;
            px = a * relx + b * rely + v1x;
            py = b * relx - a * rely + v1y;
        }
    }

    Some(clamped_tile_coord(px, py, input.width, input.height))
}

pub fn mirror_filter_left(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> bool {
    (bx - ax) * (cy - ay) > (by - ay) * (cx - ax)
}

pub fn mirror_filter_apply(input: &mut GenerateInput, source: Option<MapTile>) {
    if let Some(source) = source {
        input.floor = source.floor;
        if !source.block.synthetic {
            input.block = source.block;
        }
        input.overlay = source.overlay;
        input.packed_data = source.packed_data;
    }
}

pub fn ore_median_filter_quad_valid<F>(input: &GenerateInput, mut tile_at: F) -> bool
where
    F: FnMut(i32, i32) -> MapTile,
{
    let width = input.width;
    let height = input.height;
    ore_median_quad_valid_with(input, width, height, &mut tile_at)
}

pub fn ore_median_filter_apply<F>(
    input: &mut GenerateInput,
    radius: f32,
    percentile: f32,
    mut tile_at: F,
) where
    F: FnMut(i32, i32) -> MapTile,
{
    if input.overlay == MapBlock::SPAWN {
        return;
    }

    let width = input.width;
    let height = input.height;

    if !input.overlay.is_air() && !ore_median_quad_valid_with(input, width, height, &mut tile_at) {
        input.overlay = MapBlock::AIR;
    }

    let rad = radius as i32;
    let mut overlays = Vec::new();
    for x in -rad..=rad {
        for y in -rad..=rad {
            if x * x + y * y > rad * rad {
                continue;
            }

            let tile = sample_tile(width, height, input.x + x, input.y + y, &mut tile_at);
            if tile.overlay != MapBlock::SPAWN {
                overlays.push((tile.overlay_id(), tile.overlay));
            }
        }
    }

    if overlays.is_empty() {
        return;
    }

    overlays.sort_by_key(|(id, _)| *id);
    let index = ((overlays.len() as f32 * percentile) as usize).min(overlays.len() - 1);
    input.overlay = overlays[index].1;
}

fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}

fn vector_from_degrees(degrees: f32, length: f32) -> (f32, f32) {
    let radians = degrees.to_radians();
    (radians.cos() * length, radians.sin() * length)
}

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

fn sample_tile<F>(width: i32, height: i32, x: i32, y: i32, tile_at: &mut F) -> MapTile
where
    F: FnMut(i32, i32) -> MapTile,
{
    tile_at(
        clamp_i32(x, 0, (width - 1).max(0)),
        clamp_i32(y, 0, (height - 1).max(0)),
    )
}

fn ore_median_quad_valid_with<F>(
    input: &GenerateInput,
    width: i32,
    height: i32,
    tile_at: &mut F,
) -> bool
where
    F: FnMut(i32, i32) -> MapTile,
{
    let cx = (input.x / 2) * 2;
    let cy = (input.y / 2) * 2;
    let tiles = [
        sample_tile(width, height, cx + 1, cy, tile_at),
        sample_tile(width, height, cx, cy, tile_at),
        sample_tile(width, height, cx + 1, cy + 1, tile_at),
        sample_tile(width, height, cx, cy + 1, tile_at),
    ];

    tiles
        .iter()
        .all(|tile| tile.overlay == input.overlay && !tile.block.is_static())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASALT: MapBlock = MapBlock::floor("basalt", true);
    const WATER: MapBlock = MapBlock::floor("water", false);
    const WALL: MapBlock = MapBlock::wall("wall", false, false);
    const BOULDER: MapBlock = MapBlock::wall("boulder", false, true);
    const ORE_LEAD: MapBlock = MapBlock::overlay("ore-lead", true);

    #[test]
    fn generate_filter_helpers_match_java_names_pack_and_noise_offsets() {
        assert_eq!(filter_simple_name("NoiseFilter"), "noise");
        assert_eq!(filter_simple_name("OreMedianFilter"), "oremedian");
        assert_eq!(noise_sample_x(0), 10.0);
        assert_eq!(noise_sample_y(2, 3, 0.5), 13.5);
        assert_eq!(PackedTile::pack(1, 2, 3), 0x0001_0002_0003);
        assert_eq!(
            PackedTile::unpack(0x0001_0002_0003),
            PackedTile {
                block: 1,
                floor: 2,
                overlay: 3
            }
        );
        assert_eq!(
            deterministic_chance(10, 20, 3),
            deterministic_chance(10, 20, 3)
        );
    }

    #[test]
    fn clear_noise_terrain_and_ore_filters_follow_java_branching() {
        let mut input = GenerateInput::new(1, 2, 10, 10);
        input.floor = MapBlock::STONE;
        input.block = WALL;
        clear_filter_apply(&mut input, WALL, MapBlock::AIR, MapBlock::AIR);
        assert_eq!(input.block, MapBlock::AIR);

        input.overlay = ORE_LEAD;
        clear_filter_apply(&mut input, ORE_LEAD, MapBlock::AIR, MapBlock::AIR);
        assert_eq!(input.overlay, MapBlock::AIR);

        input.floor = MapBlock::STONE;
        clear_filter_apply(&mut input, MapBlock::STONE, ORE_LEAD, MapBlock::AIR);
        assert_eq!(input.overlay, ORE_LEAD);

        input.block = WALL;
        noise_filter_apply(&mut input, 0.9, 0.5, MapBlock::AIR, BASALT, BOULDER);
        assert_eq!(input.floor, BASALT);
        assert_eq!(input.block, BOULDER);

        let terrain = terrain_filter_noise(0.2, 0, 0, 10, 10, 2.0);
        assert!((terrain - 1.6142136).abs() < 0.00001);
        terrain_filter_apply(&mut input, terrain, 1.0, WATER, WALL);
        assert_eq!(input.floor, WATER);
        assert_eq!(input.block, WALL);

        input.floor = MapBlock::STONE;
        input.overlay = MapBlock::AIR;
        ore_filter_apply(&mut input, 0.9, 0.81, ORE_LEAD, MapBlock::AIR);
        assert_eq!(input.overlay, ORE_LEAD);
        input.overlay = MapBlock::SPAWN;
        ore_filter_apply(&mut input, 0.9, 0.81, ORE_LEAD, MapBlock::AIR);
        assert_eq!(input.overlay, MapBlock::SPAWN);
    }

    #[test]
    fn scatter_median_circle_and_blend_filters_follow_java_edges() {
        let mut input = GenerateInput::new(2, 2, 5, 5);
        input.floor = MapBlock::STONE;
        input.block = MapBlock::AIR;
        scatter_filter_apply(&mut input, 0.01, 0.013, MapBlock::AIR, BASALT, BOULDER);
        assert_eq!(input.block, BOULDER);
        assert_eq!(input.floor, BASALT);

        input.block = MapBlock::AIR;
        scatter_filter_apply(&mut input, 0.5, 0.013, MapBlock::AIR, WATER, ORE_LEAD);
        assert_ne!(input.overlay, ORE_LEAD);
        scatter_filter_apply(&mut input, 0.01, 0.013, MapBlock::AIR, WATER, ORE_LEAD);
        assert_eq!(input.overlay, ORE_LEAD);

        let mut values = [9, 1, 5, 2];
        assert_eq!(median_filter_pick(&mut values, 0.5), Some(5));
        let offsets = circle_offsets(1);
        assert_eq!(offsets, vec![(-1, 0), (0, -1), (0, 0), (0, 1), (1, 0)]);

        input.floor = MapBlock::STONE;
        input.block = MapBlock::AIR;
        assert!(!blend_filter_should_skip(
            &input,
            BASALT,
            WATER,
            MapBlock::AIR
        ));
        blend_filter_apply(&mut input, true, WATER);
        assert_eq!(input.floor, WATER);
        blend_filter_apply(&mut input, true, BOULDER);
        assert_eq!(input.block, BOULDER);
        assert!(blend_filter_should_skip(
            &input,
            MapBlock::AIR,
            WATER,
            MapBlock::AIR
        ));
    }

    #[test]
    fn river_distort_and_mirror_filters_follow_buffered_java_edges() {
        let mut input = GenerateInput::new(2, 2, 5, 5);
        input.floor = MapBlock::STONE;
        input.block = WALL;

        river_noise_filter_apply(
            &mut input,
            0.11,
            0.0,
            0.1,
            WATER,
            BASALT,
            BOULDER,
            MapBlock::AIR,
        );
        assert_eq!(input.floor, BASALT);
        assert_eq!(input.block, BOULDER);

        let before = input;
        river_noise_filter_apply(
            &mut input,
            -0.1,
            0.0,
            0.1,
            WATER,
            BASALT,
            BOULDER,
            MapBlock::AIR,
        );
        assert_eq!(input, before);

        let source = MapTile::new(BASALT, WALL, ORE_LEAD, 99);
        let mut synthetic_input = GenerateInput::new(0, 0, 4, 4);
        synthetic_input.block = MapBlock::wall("synthetic-wall", true, false);
        distort_filter_apply(&mut synthetic_input, source);
        assert_eq!(synthetic_input.floor, BASALT);
        assert_eq!(synthetic_input.block.name, "synthetic-wall");
        assert_eq!(synthetic_input.overlay, ORE_LEAD);

        let mut normal_input = GenerateInput::new(0, 0, 4, 4);
        distort_filter_apply(&mut normal_input, source);
        assert_eq!(normal_input.block, WALL);
        assert_eq!(
            distort_filter_source_coord(&GenerateInput::new(1, 1, 4, 4), 5.0, -3.0, 4.0),
            (3, 0)
        );

        let left_half = GenerateInput::new(0, 1, 4, 4);
        assert_eq!(mirror_filter_source_coord(&left_half, 0, false), None);
        let right_half = GenerateInput::new(3, 1, 4, 4);
        assert_eq!(
            mirror_filter_source_coord(&right_half, 0, false),
            Some((0, 1))
        );

        let skewed = GenerateInput::new(5, 0, 6, 4);
        assert_eq!(mirror_filter_source_coord(&skewed, 45, false), Some((0, 3)));

        let mut mirrored = GenerateInput::new(3, 1, 4, 4);
        let source = MapTile::new(WATER, BOULDER, ORE_LEAD, 1234);
        mirror_filter_apply(&mut mirrored, Some(source));
        assert_eq!(mirrored.floor, WATER);
        assert_eq!(mirrored.block, BOULDER);
        assert_eq!(mirrored.overlay, ORE_LEAD);
        assert_eq!(mirrored.packed_data, 1234);
    }

    #[test]
    fn ore_median_filter_keeps_spawn_quad_and_percentile_rules() {
        const COPPER_ID: i16 = 10;
        const LEAD_ID: i16 = 20;
        const SPAWN_ID: i16 = 30;
        const AIR_ID: i16 = 0;

        fn tile_for(x: i32, y: i32, center_overlay: MapBlock, center_block: MapBlock) -> MapTile {
            let (overlay, id) = match (x, y) {
                (2, 2) | (3, 2) | (2, 3) | (3, 3) => (center_overlay, COPPER_ID),
                (1, 2) | (2, 1) | (2, 4) | (4, 2) => (ORE_LEAD, LEAD_ID),
                (3, 1) => (MapBlock::SPAWN, SPAWN_ID),
                _ => (MapBlock::AIR, AIR_ID),
            };
            MapTile::new(
                MapBlock::STONE,
                if (x, y) == (3, 2) {
                    center_block
                } else {
                    MapBlock::AIR
                },
                overlay,
                PackedTile::pack(0, 1, id),
            )
        }

        let mut input = GenerateInput::new(2, 2, 6, 6);
        input.overlay = ORE_LEAD;
        ore_median_filter_apply(&mut input, 1.0, 1.0, |x, y| {
            tile_for(x, y, ORE_LEAD, MapBlock::AIR)
        });
        assert_eq!(input.overlay, ORE_LEAD);

        input.overlay = ORE_LEAD;
        ore_median_filter_apply(&mut input, 1.0, 0.0, |x, y| tile_for(x, y, ORE_LEAD, WALL));
        assert_eq!(input.overlay, ORE_LEAD);

        input.overlay = ORE_LEAD;
        ore_median_filter_apply(&mut input, 0.0, 0.0, |x, y| {
            tile_for(x, y, MapBlock::AIR, WALL)
        });
        assert_eq!(input.overlay, MapBlock::AIR);

        input.overlay = MapBlock::SPAWN;
        ore_median_filter_apply(&mut input, 1.0, 0.0, |x, y| tile_for(x, y, ORE_LEAD, WALL));
        assert_eq!(input.overlay, MapBlock::SPAWN);

        input.overlay = MapBlock::AIR;
        ore_median_filter_apply(&mut input, 1.0, 0.8, |x, y| {
            tile_for(x, y, MapBlock::AIR, MapBlock::AIR)
        });
        assert_eq!(input.overlay, ORE_LEAD);
    }
}
