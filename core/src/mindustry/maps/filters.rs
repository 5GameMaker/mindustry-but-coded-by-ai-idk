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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
}

impl TilePos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostFilterTile {
    pub pos: TilePos,
    pub floor: MapBlock,
    pub block: MapBlock,
    pub overlay: MapBlock,
    pub team: i32,
    pub is_core: bool,
    pub is_center: bool,
    pub is_storage: bool,
    pub synthetic: bool,
    pub item_capacity: i32,
}

impl PostFilterTile {
    pub fn new(pos: TilePos) -> Self {
        Self {
            pos,
            floor: MapBlock::STONE,
            block: MapBlock::AIR,
            overlay: MapBlock::AIR,
            team: 0,
            is_core: false,
            is_center: false,
            is_storage: false,
            synthetic: false,
            item_capacity: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockWrite {
    pub pos: TilePos,
    pub block: MapBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStackSpec {
    pub item_id: i16,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemGrant {
    pub pos: TilePos,
    pub item_id: i16,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicFilterPlan {
    pub code: Option<String>,
    pub max_instructions: usize,
    pub looped: bool,
    pub update_logic_vars_first: bool,
}

pub const LOGIC_FILTER_MAX_INSTRUCTIONS_EXECUTION: usize = 500 * 500 * 25;

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

pub fn post_filter_trim_removals(
    candidates: &[TilePos],
    amount: usize,
    shuffled_indices: &[usize],
) -> Vec<TilePos> {
    if amount >= candidates.len() {
        return Vec::new();
    }

    let mut ordered = Vec::with_capacity(candidates.len());
    let mut seen = vec![false; candidates.len()];

    for &index in shuffled_indices {
        if index < candidates.len() && !seen[index] {
            seen[index] = true;
            ordered.push(candidates[index]);
        }
    }

    for (index, pos) in candidates.iter().enumerate() {
        if !seen[index] {
            ordered.push(*pos);
        }
    }

    ordered.into_iter().skip(amount).collect()
}

pub fn core_spawn_filter_removals(
    tiles: &[PostFilterTile],
    default_team: i32,
    amount: usize,
    shuffled_indices: &[usize],
) -> Vec<TilePos> {
    let candidates: Vec<_> = tiles
        .iter()
        .filter(|tile| tile.team == default_team && tile.is_core && tile.is_center)
        .map(|tile| tile.pos)
        .collect();
    post_filter_trim_removals(&candidates, amount, shuffled_indices)
}

pub fn enemy_spawn_filter_clears(
    tiles: &[PostFilterTile],
    amount: usize,
    shuffled_indices: &[usize],
) -> Vec<TilePos> {
    let candidates: Vec<_> = tiles
        .iter()
        .filter(|tile| tile.overlay == MapBlock::SPAWN)
        .map(|tile| tile.pos)
        .collect();
    post_filter_trim_removals(&candidates, amount, shuffled_indices)
}

pub fn spawn_path_filter_points(
    tiles: &[PostFilterTile],
    wave_team: i32,
) -> (Vec<TilePos>, Vec<TilePos>) {
    let mut cores = Vec::new();
    let mut spawns = Vec::new();

    for tile in tiles {
        if tile.overlay == MapBlock::SPAWN {
            spawns.push(tile.pos);
        }
        if tile.is_core && tile.team != wave_team {
            cores.push(tile.pos);
        }
    }

    (cores, spawns)
}

pub fn expand_spawn_path_walls<F>(
    path: &[TilePos],
    radius: i32,
    width: i32,
    height: i32,
    block: MapBlock,
    mut is_synthetic: F,
) -> Vec<BlockWrite>
where
    F: FnMut(TilePos) -> bool,
{
    let mut writes = Vec::new();
    let rad = radius.max(0);

    for tile in path {
        for x in -rad..=rad {
            for y in -rad..=rad {
                let pos = TilePos::new(tile.x + x, tile.y + y);
                if in_bounds(pos, width, height)
                    && within_radius_i32(x, y, rad)
                    && !is_synthetic(pos)
                {
                    writes.push(BlockWrite { pos, block });
                }
            }
        }
    }

    writes
}

pub fn random_item_filter_grants(
    tiles: &[PostFilterTile],
    drops: &[ItemStackSpec],
    chance: f32,
    rolls: &[(f32, i32)],
) -> Vec<ItemGrant> {
    let mut grants = Vec::new();
    let mut roll_index = 0usize;

    for tile in tiles {
        if !tile.is_storage || tile.is_core {
            continue;
        }

        for drop in drops {
            let (chance_roll, amount_roll) = rolls.get(roll_index).copied().unwrap_or((1.0, 0));
            roll_index += 1;

            if random_item_chance_passes(chance_roll, chance) {
                let amount = amount_roll.clamp(0, drop.amount.max(0));
                let amount = amount.min(tile.item_capacity.max(0));
                grants.push(ItemGrant {
                    pos: tile.pos,
                    item_id: drop.item_id,
                    amount,
                });
            }
        }
    }

    grants
}

pub fn random_item_chance_passes(roll: f32, chance: f32) -> bool {
    if chance <= 0.0 {
        false
    } else if chance >= 1.0 {
        true
    } else {
        roll < chance
    }
}

pub fn logic_filter_plan(code: Option<&str>, looped: bool) -> LogicFilterPlan {
    LogicFilterPlan {
        code: code.map(ToOwned::to_owned),
        max_instructions: LOGIC_FILTER_MAX_INSTRUCTIONS_EXECUTION,
        looped,
        update_logic_vars_first: true,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoiseFilter {
    pub seed: i32,
    pub scl: f32,
    pub threshold: f32,
    pub octaves: f32,
    pub falloff: f32,
    pub tilt: f32,
    pub floor: MapBlock,
    pub block: MapBlock,
    pub target: MapBlock,
}

impl Default for NoiseFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            scl: 40.0,
            threshold: 0.5,
            octaves: 3.0,
            falloff: 0.5,
            tilt: 0.0,
            floor: MapBlock::STONE,
            block: MapBlock::STONE_WALL,
            target: MapBlock::AIR,
        }
    }
}

impl NoiseFilter {
    pub fn simple_name(&self) -> &'static str {
        "noise"
    }

    pub fn is_buffered(&self) -> bool {
        false
    }

    pub fn apply(&self, input: &mut GenerateInput) {
        let noise = terrain_filter_noise(
            deterministic_chance(input.x, input.y, self.seed),
            input.x,
            input.y,
            input.width,
            input.height,
            self.scl,
        );
        noise_filter_apply(
            input,
            noise,
            self.threshold,
            self.target,
            self.floor,
            self.block,
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScatterFilter {
    pub seed: i32,
    pub chance: f32,
    pub flooronto: MapBlock,
    pub floor: MapBlock,
    pub block: MapBlock,
}

impl Default for ScatterFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            chance: 0.013,
            flooronto: MapBlock::AIR,
            floor: MapBlock::AIR,
            block: MapBlock::AIR,
        }
    }
}

impl ScatterFilter {
    pub fn simple_name(&self) -> &'static str {
        "scatter"
    }

    pub fn apply(&self, input: &mut GenerateInput) {
        scatter_filter_apply(
            input,
            deterministic_chance(input.x, input.y, self.seed),
            self.chance,
            self.flooronto,
            self.floor,
            self.block,
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainFilter {
    pub seed: i32,
    pub scl: f32,
    pub threshold: f32,
    pub octaves: f32,
    pub falloff: f32,
    pub magnitude: f32,
    pub circle_scl: f32,
    pub tilt: f32,
    pub floor: MapBlock,
    pub block: MapBlock,
}

impl Default for TerrainFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            scl: 40.0,
            threshold: 0.9,
            octaves: 3.0,
            falloff: 0.5,
            magnitude: 1.0,
            circle_scl: 2.1,
            tilt: 0.0,
            floor: MapBlock::AIR,
            block: MapBlock::STONE_WALL,
        }
    }
}

impl TerrainFilter {
    pub fn simple_name(&self) -> &'static str {
        "terrain"
    }

    pub fn apply(&self, input: &mut GenerateInput) {
        let noise = terrain_filter_noise(
            deterministic_chance(input.x, input.y, self.seed),
            input.x,
            input.y,
            input.width,
            input.height,
            self.circle_scl,
        );
        terrain_filter_apply(input, noise, self.threshold, self.floor, self.block);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistortFilter {
    pub seed: i32,
    pub scl: f32,
    pub mag: f32,
}

impl Default for DistortFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            scl: 40.0,
            mag: 5.0,
        }
    }
}

impl DistortFilter {
    pub fn simple_name(&self) -> &'static str {
        "distort"
    }

    pub fn apply<F>(&self, input: &mut GenerateInput, mut source_at: F)
    where
        F: FnMut(i32, i32) -> MapTile,
    {
        let random = deterministic_chance(input.x, input.y, self.seed);
        let (sx, sy) = distort_filter_source_coord(
            input,
            random * self.scl,
            deterministic_chance(input.y, input.x, self.seed.wrapping_add(1)) * self.scl,
            self.mag,
        );
        let source = source_at(sx, sy);
        distort_filter_apply(input, source);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RiverNoiseFilter {
    pub seed: i32,
    pub scl: f32,
    pub threshold: f32,
    pub threshold2: f32,
    pub octaves: f32,
    pub falloff: f32,
    pub floor: MapBlock,
    pub floor2: MapBlock,
    pub block: MapBlock,
    pub target: MapBlock,
}

impl Default for RiverNoiseFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            scl: 40.0,
            threshold: 0.0,
            threshold2: 0.1,
            octaves: 1.0,
            falloff: 0.5,
            floor: MapBlock::AIR,
            floor2: MapBlock::AIR,
            block: MapBlock::STONE_WALL,
            target: MapBlock::AIR,
        }
    }
}

impl RiverNoiseFilter {
    pub fn simple_name(&self) -> &'static str {
        "rivernoise"
    }

    pub fn apply(&self, input: &mut GenerateInput) {
        let noise = terrain_filter_noise(
            deterministic_chance(input.x, input.y, self.seed),
            input.x,
            input.y,
            input.width,
            input.height,
            self.scl,
        );
        river_noise_filter_apply(
            input,
            noise,
            self.threshold,
            self.threshold2,
            self.floor,
            self.floor2,
            self.block,
            self.target,
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OreFilter {
    pub seed: i32,
    pub scl: f32,
    pub threshold: f32,
    pub octaves: f32,
    pub falloff: f32,
    pub tilt: f32,
    pub ore: MapBlock,
    pub target: MapBlock,
}

impl Default for OreFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            scl: 23.0,
            threshold: 0.81,
            octaves: 2.0,
            falloff: 0.3,
            tilt: 0.0,
            ore: MapBlock::ORE_COPPER,
            target: MapBlock::AIR,
        }
    }
}

impl OreFilter {
    pub fn simple_name(&self) -> &'static str {
        "ore"
    }

    pub fn apply(&self, input: &mut GenerateInput) {
        let noise = deterministic_chance(input.x, input.y, self.seed);
        ore_filter_apply(input, noise, self.threshold, self.ore, self.target);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OreMedianFilter {
    pub seed: i32,
    pub radius: f32,
    pub percentile: f32,
}

impl Default for OreMedianFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            radius: 2.0,
            percentile: 0.5,
        }
    }
}

impl OreMedianFilter {
    pub fn simple_name(&self) -> &'static str {
        "oremedian"
    }

    pub fn is_buffered(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MedianFilter {
    pub seed: i32,
    pub radius: f32,
    pub percentile: f32,
}

impl Default for MedianFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            radius: 2.0,
            percentile: 0.5,
        }
    }
}

impl MedianFilter {
    pub fn simple_name(&self) -> &'static str {
        "median"
    }

    pub fn is_buffered(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlendFilter {
    pub seed: i32,
    pub radius: f32,
    pub block: MapBlock,
    pub floor: MapBlock,
    pub ignore: MapBlock,
}

impl Default for BlendFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            radius: 2.0,
            block: MapBlock::SAND,
            floor: MapBlock::SAND_WATER,
            ignore: MapBlock::AIR,
        }
    }
}

impl BlendFilter {
    pub fn simple_name(&self) -> &'static str {
        "blend"
    }

    pub fn is_buffered(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirrorFilter {
    pub seed: i32,
    pub angle: i32,
    pub rotate: bool,
}

impl Default for MirrorFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            angle: 45,
            rotate: false,
        }
    }
}

impl MirrorFilter {
    pub fn simple_name(&self) -> &'static str {
        "mirror"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClearFilter {
    pub seed: i32,
    pub target: MapBlock,
    pub replace: MapBlock,
    pub ignore: MapBlock,
}

impl Default for ClearFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            target: MapBlock::STONE,
            replace: MapBlock::AIR,
            ignore: MapBlock::AIR,
        }
    }
}

impl ClearFilter {
    pub fn simple_name(&self) -> &'static str {
        "clear"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreSpawnFilter {
    pub seed: i32,
    pub amount: i32,
}

impl Default for CoreSpawnFilter {
    fn default() -> Self {
        Self { seed: 0, amount: 1 }
    }
}

impl CoreSpawnFilter {
    pub fn simple_name(&self) -> &'static str {
        "corespawn"
    }

    pub fn is_post(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnemySpawnFilter {
    pub seed: i32,
    pub amount: i32,
}

impl Default for EnemySpawnFilter {
    fn default() -> Self {
        Self { seed: 0, amount: 1 }
    }
}

impl EnemySpawnFilter {
    pub fn simple_name(&self) -> &'static str {
        "enemyspawn"
    }

    pub fn is_post(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnPathFilter {
    pub seed: i32,
    pub radius: i32,
    pub block: MapBlock,
}

impl Default for SpawnPathFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            radius: 3,
            block: MapBlock::AIR,
        }
    }
}

impl SpawnPathFilter {
    pub fn simple_name(&self) -> &'static str {
        "spawnpath"
    }

    pub fn is_post(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicFilter {
    pub seed: i32,
    pub code: Option<String>,
    pub looped: bool,
}

impl Default for LogicFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            code: None,
            looped: false,
        }
    }
}

impl LogicFilter {
    pub fn simple_name(&self) -> &'static str {
        "logic"
    }

    pub fn is_post(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RandomItemFilter {
    pub seed: i32,
    pub drops: Vec<ItemStackSpec>,
    pub chance: f32,
}

impl Default for RandomItemFilter {
    fn default() -> Self {
        Self {
            seed: 0,
            drops: Vec::new(),
            chance: 0.3,
        }
    }
}

impl RandomItemFilter {
    pub fn simple_name(&self) -> &'static str {
        "randomitem"
    }

    pub fn is_post(&self) -> bool {
        true
    }
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

fn in_bounds(pos: TilePos, width: i32, height: i32) -> bool {
    pos.x >= 0 && pos.y >= 0 && pos.x < width && pos.y < height
}

fn within_radius_i32(x: i32, y: i32, radius: i32) -> bool {
    x * x + y * y <= radius * radius
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

    #[test]
    fn post_spawn_filters_trim_candidates_like_java_post_filters() {
        fn tile(pos: TilePos) -> PostFilterTile {
            PostFilterTile::new(pos)
        }

        let mut tiles = vec![
            tile(TilePos::new(0, 0)),
            tile(TilePos::new(1, 0)),
            tile(TilePos::new(2, 0)),
            tile(TilePos::new(3, 0)),
            tile(TilePos::new(4, 0)),
        ];

        tiles[0].is_core = true;
        tiles[0].is_center = true;
        tiles[0].team = 1;
        tiles[1].is_core = true;
        tiles[1].is_center = false;
        tiles[1].team = 1;
        tiles[2].is_core = true;
        tiles[2].is_center = true;
        tiles[2].team = 2;
        tiles[3].is_core = true;
        tiles[3].is_center = true;
        tiles[3].team = 1;
        tiles[4].is_core = true;
        tiles[4].is_center = true;
        tiles[4].team = 1;

        assert_eq!(
            core_spawn_filter_removals(&tiles, 1, 1, &[2, 0, 1]),
            vec![TilePos::new(0, 0), TilePos::new(3, 0)]
        );
        assert!(core_spawn_filter_removals(&tiles, 1, 3, &[2, 0, 1]).is_empty());

        tiles[1].overlay = MapBlock::SPAWN;
        tiles[3].overlay = MapBlock::SPAWN;
        tiles[4].overlay = MapBlock::SPAWN;
        assert_eq!(
            enemy_spawn_filter_clears(&tiles, 2, &[1, 0, 2]),
            vec![TilePos::new(4, 0)]
        );
        assert_eq!(
            enemy_spawn_filter_clears(&tiles, 0, &[2, 0, 1]),
            vec![TilePos::new(4, 0), TilePos::new(1, 0), TilePos::new(3, 0)]
        );
    }

    #[test]
    fn spawn_path_random_item_and_logic_post_rules_are_plannable() {
        let mut tiles = vec![
            PostFilterTile::new(TilePos::new(0, 0)),
            PostFilterTile::new(TilePos::new(1, 1)),
            PostFilterTile::new(TilePos::new(2, 2)),
            PostFilterTile::new(TilePos::new(3, 3)),
        ];
        tiles[0].is_core = true;
        tiles[0].team = 1;
        tiles[1].is_core = true;
        tiles[1].team = 2;
        tiles[2].overlay = MapBlock::SPAWN;

        let (cores, spawns) = spawn_path_filter_points(&tiles, 2);
        assert_eq!(cores, vec![TilePos::new(0, 0)]);
        assert_eq!(spawns, vec![TilePos::new(2, 2)]);

        let writes = expand_spawn_path_walls(&[TilePos::new(1, 1)], 1, 3, 3, WALL, |pos| {
            pos == TilePos::new(1, 0)
        });
        assert_eq!(
            writes,
            vec![
                BlockWrite {
                    pos: TilePos::new(0, 1),
                    block: WALL
                },
                BlockWrite {
                    pos: TilePos::new(1, 1),
                    block: WALL
                },
                BlockWrite {
                    pos: TilePos::new(1, 2),
                    block: WALL
                },
                BlockWrite {
                    pos: TilePos::new(2, 1),
                    block: WALL
                }
            ]
        );

        let mut storage = PostFilterTile::new(TilePos::new(5, 5));
        storage.is_storage = true;
        storage.item_capacity = 4;
        let mut core_storage = PostFilterTile::new(TilePos::new(6, 5));
        core_storage.is_storage = true;
        core_storage.is_core = true;
        core_storage.item_capacity = 100;
        let drops = [
            ItemStackSpec {
                item_id: 7,
                amount: 10,
            },
            ItemStackSpec {
                item_id: 8,
                amount: 3,
            },
        ];
        let grants = random_item_filter_grants(
            &[storage, core_storage],
            &drops,
            0.5,
            &[(0.49, 9), (0.51, 2)],
        );
        assert_eq!(
            grants,
            vec![ItemGrant {
                pos: TilePos::new(5, 5),
                item_id: 7,
                amount: 4
            }]
        );
        assert!(
            random_item_filter_grants(&[storage], &drops, 0.0, &[(0.0, 1), (0.0, 1)]).is_empty()
        );

        let plan = logic_filter_plan(Some("print \"hi\""), true);
        assert_eq!(plan.code.as_deref(), Some("print \"hi\""));
        assert_eq!(
            plan.max_instructions,
            LOGIC_FILTER_MAX_INSTRUCTIONS_EXECUTION
        );
        assert!(plan.looped);
        assert!(plan.update_logic_vars_first);
    }

    #[test]
    fn filter_data_defaults_match_upstream_field_initializers() {
        let noise = NoiseFilter::default();
        assert_eq!(noise.scl, 40.0);
        assert_eq!(noise.threshold, 0.5);
        assert_eq!(noise.octaves, 3.0);
        assert_eq!(noise.floor, MapBlock::STONE);
        assert_eq!(noise.block, MapBlock::STONE_WALL);
        assert_eq!(noise.target, MapBlock::AIR);

        let scatter = ScatterFilter::default();
        assert_eq!(scatter.chance, 0.013);
        assert_eq!(scatter.flooronto, MapBlock::AIR);

        let terrain = TerrainFilter::default();
        assert_eq!(terrain.magnitude, 1.0);
        assert_eq!(terrain.circle_scl, 2.1);

        let river = RiverNoiseFilter::default();
        assert_eq!(river.threshold2, 0.1);
        assert_eq!(river.floor2, MapBlock::AIR);
        assert_eq!(river.block, MapBlock::STONE_WALL);

        let ore = OreFilter::default();
        assert_eq!(ore.scl, 23.0);
        assert_eq!(ore.threshold, 0.81);
        assert_eq!(ore.ore, MapBlock::ORE_COPPER);

        let blend = BlendFilter::default();
        assert_eq!(blend.radius, 2.0);
        assert_eq!(blend.block, MapBlock::SAND);

        let mirror = MirrorFilter::default();
        assert_eq!(mirror.angle, 45);
        assert!(!mirror.rotate);

        let clear = ClearFilter::default();
        assert_eq!(clear.target, MapBlock::STONE);
        assert_eq!(clear.replace, MapBlock::AIR);

        let core_spawn = CoreSpawnFilter::default();
        assert_eq!(core_spawn.amount, 1);
        assert!(core_spawn.is_post());

        let enemy_spawn = EnemySpawnFilter::default();
        assert_eq!(enemy_spawn.amount, 1);
        assert!(enemy_spawn.is_post());

        let spawn_path = SpawnPathFilter::default();
        assert_eq!(spawn_path.radius, 3);
        assert_eq!(spawn_path.block, MapBlock::AIR);
        assert!(spawn_path.is_post());

        let logic = LogicFilter::default();
        assert!(logic.code.is_none());
        assert!(!logic.looped);
        assert!(logic.is_post());

        let random_item = RandomItemFilter::default();
        assert!(random_item.drops.is_empty());
        assert_eq!(random_item.chance, 0.3);
        assert!(random_item.is_post());
    }
}
