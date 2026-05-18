use std::io::{self, Read, Write};

use crate::mindustry::{
    ctype::ContentId,
    world::{point2_pack, point2_x, point2_y, BlockId},
};

pub const DEFAULT_PAYLOAD_SPEED: f32 = 0.7;
pub const DEFAULT_PAYLOAD_ROTATE_SPEED: f32 = 5.0;
pub const PAYLOAD_BLOCK_TYPE: u8 = 0;
pub const PAYLOAD_UNIT_TYPE: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn dst(self, other: Vec2) -> f32 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
        .len()
    }

    pub fn clamp_rect(self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            x: self.x.clamp(min_x, max_x),
            y: self.y.clamp(min_y, max_y),
        }
    }

    pub fn approach(self, target: Vec2, amount: f32) -> Self {
        let delta = Vec2 {
            x: target.x - self.x,
            y: target.y - self.y,
        };
        let len = delta.len();
        if len <= amount || len <= f32::EPSILON {
            target
        } else {
            Self {
                x: self.x + delta.x / len * amount,
                y: self.y + delta.y / len * amount,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadRef {
    Block {
        block: BlockId,
        version: u8,
        build_bytes: Vec<u8>,
    },
    Unit {
        class_id: u8,
        unit_bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadBlockBuildState {
    pub payload: Option<PayloadRef>,
    pub pay_vector: Vec2,
    pub pay_rotation: f32,
    pub carried: bool,
}

impl Default for PayloadBlockBuildState {
    fn default() -> Self {
        Self {
            payload: None,
            pay_vector: Vec2::ZERO,
            pay_rotation: 0.0,
            carried: false,
        }
    }
}

pub fn payload_block_accept_payload(state: &PayloadBlockBuildState) -> bool {
    state.payload.is_none()
}

pub fn payload_block_handle_payload(
    state: &mut PayloadBlockBuildState,
    payload: PayloadRef,
    build_pos: Vec2,
    source_pos: Vec2,
    source_rotation: f32,
    size: i32,
    tile_size: f32,
) {
    state.payload = Some(payload);
    state.pay_vector = payload_offset_from_source(build_pos, source_pos, size, tile_size);
    state.pay_rotation = source_rotation;
}

pub fn payload_block_take_payload(state: &mut PayloadBlockBuildState) -> Option<PayloadRef> {
    state.payload.take()
}

pub fn payload_block_picked_up(state: &mut PayloadBlockBuildState) {
    state.carried = true;
}

pub fn payload_block_draw_team_top(state: &mut PayloadBlockBuildState) {
    state.carried = false;
}

pub fn payload_offset_from_source(
    build_pos: Vec2,
    source_pos: Vec2,
    size: i32,
    tile_size: f32,
) -> Vec2 {
    let half = size as f32 * tile_size / 2.0;
    Vec2 {
        x: source_pos.x - build_pos.x,
        y: source_pos.y - build_pos.y,
    }
    .clamp_rect(-half, -half, half, half)
}

pub fn payload_block_move_in(
    state: &mut PayloadBlockBuildState,
    rotate_payload: bool,
    block_rotate: bool,
    rotdeg: f32,
    payload_speed: f32,
    payload_rotate_speed: f32,
    delta: f32,
) -> bool {
    if state.payload.is_none() {
        return false;
    }

    if rotate_payload {
        let target = if block_rotate { rotdeg } else { 90.0 };
        state.pay_rotation =
            move_toward_angle(state.pay_rotation, target, payload_rotate_speed * delta);
    }
    state.pay_vector = state.pay_vector.approach(Vec2::ZERO, payload_speed * delta);
    payload_block_has_arrived(state)
}

pub fn payload_block_move_out_target(rotdeg: f32, size: i32, tile_size: f32) -> Vec2 {
    let length = size as f32 * tile_size / 2.0;
    let radians = rotdeg.to_radians();
    Vec2 {
        x: radians.cos() * length,
        y: radians.sin() * length,
    }
}

pub fn payload_block_move_out_step(
    state: &mut PayloadBlockBuildState,
    rotdeg: f32,
    size: i32,
    tile_size: f32,
    payload_speed: f32,
    payload_rotate_speed: f32,
    delta: f32,
) -> bool {
    if state.payload.is_none() {
        return false;
    }

    let dest = payload_block_move_out_target(rotdeg, size, tile_size);
    state.pay_rotation =
        move_toward_angle(state.pay_rotation, rotdeg, payload_rotate_speed * delta);
    state.pay_vector = state.pay_vector.approach(dest, payload_speed * delta);
    state.pay_vector.dst(dest) <= 0.001
}

pub fn payload_block_has_arrived(state: &PayloadBlockBuildState) -> bool {
    state.pay_vector.len() <= 0.01
}

pub fn write_payload_block_build_common<W: Write>(
    write: &mut W,
    state: &PayloadBlockBuildState,
) -> io::Result<()> {
    write_f32(write, state.pay_vector.x)?;
    write_f32(write, state.pay_vector.y)?;
    write_f32(write, state.pay_rotation)?;
    write_payload_ref(write, state.payload.as_ref())
}

pub fn read_empty_payload_block_build_common<R: Read>(
    read: &mut R,
) -> io::Result<PayloadBlockBuildState> {
    let pay_vector = Vec2 {
        x: read_f32(read)?,
        y: read_f32(read)?,
    };
    let pay_rotation = read_f32(read)?;
    if read_bool(read)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "non-empty payload body requires block/unit codec",
        ));
    }
    Ok(PayloadBlockBuildState {
        payload: None,
        pay_vector,
        pay_rotation,
        carried: false,
    })
}

pub fn write_payload_ref<W: Write>(write: &mut W, payload: Option<&PayloadRef>) -> io::Result<()> {
    write_bool(write, payload.is_some())?;
    match payload {
        None => Ok(()),
        Some(PayloadRef::Block {
            block,
            version,
            build_bytes,
        }) => {
            write_u8(write, PAYLOAD_BLOCK_TYPE)?;
            write_i16(write, *block)?;
            write_u8(write, *version)?;
            write.write_all(build_bytes)
        }
        Some(PayloadRef::Unit {
            class_id,
            unit_bytes,
        }) => {
            write_u8(write, PAYLOAD_UNIT_TYPE)?;
            write_u8(write, *class_id)?;
            write.write_all(unit_bytes)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PayloadSourceState {
    pub unit: Option<ContentId>,
    pub config_block: Option<BlockId>,
    pub command_pos: Option<Vec2>,
    pub has_payload: bool,
    pub scl: f32,
}

impl Default for PayloadSourceState {
    fn default() -> Self {
        Self {
            unit: None,
            config_block: None,
            command_pos: None,
            has_payload: false,
            scl: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadSourceSpawn {
    None,
    Unit(ContentId),
    Block(BlockId),
}

pub fn payload_source_configure_unit(state: &mut PayloadSourceState, unit: ContentId) {
    if state.unit != Some(unit) {
        state.unit = Some(unit);
        state.config_block = None;
        state.has_payload = false;
        state.scl = 0.0;
    }
}

pub fn payload_source_configure_block(state: &mut PayloadSourceState, block: BlockId) {
    if state.config_block != Some(block) {
        state.config_block = Some(block);
        state.unit = None;
        state.has_payload = false;
        state.scl = 0.0;
    }
}

pub fn payload_source_clear_config(state: &mut PayloadSourceState) {
    state.unit = None;
    state.config_block = None;
    state.has_payload = false;
    state.scl = 0.0;
}

pub fn payload_source_update(state: &mut PayloadSourceState) -> PayloadSourceSpawn {
    let spawn = if state.has_payload {
        PayloadSourceSpawn::None
    } else if let Some(unit) = state.unit {
        state.has_payload = true;
        PayloadSourceSpawn::Unit(unit)
    } else if let Some(block) = state.config_block {
        state.has_payload = true;
        PayloadSourceSpawn::Block(block)
    } else {
        PayloadSourceSpawn::None
    };
    state.scl = lerp_delta(state.scl, 1.0, 0.1);
    spawn
}

pub fn payload_source_accept_payload() -> bool {
    false
}

pub fn write_payload_source_extra<W: Write>(
    write: &mut W,
    unit: Option<ContentId>,
    config_block: Option<BlockId>,
    command_pos: Option<Vec2>,
) -> io::Result<()> {
    write_i16(write, unit.unwrap_or(-1))?;
    write_i16(write, config_block.unwrap_or(-1))?;
    write_vec_nullable(write, command_pos)
}

pub fn read_payload_source_extra<R: Read>(
    read: &mut R,
    revision: u8,
) -> io::Result<(Option<ContentId>, Option<BlockId>, Option<Vec2>)> {
    let unit = read_optional_i16(read)?;
    let block = read_optional_i16(read)?;
    let command_pos = if revision >= 1 {
        read_vec_nullable(read)?
    } else {
        None
    };
    Ok((unit, block, command_pos))
}

pub fn payload_void_accept_unit_payload() -> bool {
    true
}

pub fn payload_void_update(arrived: bool, efficiency: f32, has_payload: bool) -> bool {
    arrived && efficiency > 0.0 && has_payload
}

pub fn constructor_configure(
    current_recipe: &mut Option<BlockId>,
    progress: &mut f32,
    block: BlockId,
    can_produce: bool,
) {
    if *current_recipe != Some(block) {
        *progress = 0.0;
    }
    if can_produce {
        *current_recipe = Some(block);
    }
}

pub fn constructor_clear(current_recipe: &mut Option<BlockId>) {
    *current_recipe = None;
}

pub fn write_constructor_recipe<W: Write>(
    write: &mut W,
    recipe: Option<BlockId>,
) -> io::Result<()> {
    write_i16(write, recipe.unwrap_or(-1))
}

pub fn read_constructor_recipe<R: Read>(read: &mut R) -> io::Result<Option<BlockId>> {
    read_optional_i16(read)
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadDeconstructorState {
    pub progress: f32,
    pub accum: Option<Vec<f32>>,
    pub has_payload: bool,
    pub has_deconstructing: bool,
}

impl Default for PayloadDeconstructorState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            accum: None,
            has_payload: false,
            has_deconstructing: false,
        }
    }
}

pub fn payload_deconstructor_accept_payload(
    state: &PayloadDeconstructorState,
    requirements_len: usize,
    payload_size: f32,
    max_payload_size: f32,
) -> bool {
    !state.has_deconstructing
        && !state.has_payload
        && requirements_len > 0
        && payload_size <= max_payload_size
}

pub fn payload_deconstructor_begin_if_arrived(
    state: &mut PayloadDeconstructorState,
    arrived: bool,
    requirements_len: usize,
) -> bool {
    if arrived && state.has_payload {
        state.accum = Some(vec![0.0; requirements_len]);
        state.has_deconstructing = true;
        state.has_payload = false;
        state.progress = 0.0;
        true
    } else {
        false
    }
}

pub fn payload_deconstructor_should_consume(has_deconstructing: bool, enabled: bool) -> bool {
    has_deconstructing && enabled
}

#[derive(Debug, Clone, PartialEq)]
pub struct PayloadDeconstructorProgressStep {
    pub can_progress: bool,
    pub finished: bool,
    pub items_added: Vec<i32>,
}

pub fn payload_deconstructor_update_progress(
    progress: &mut f32,
    time: &mut f32,
    accum: &mut [f32],
    requirement_amounts: &[i32],
    items_total: &mut i32,
    item_capacity: i32,
    edelta: f32,
    deconstruct_speed: f32,
    build_time: f32,
    cost_multiplier: f32,
) -> PayloadDeconstructorProgressStep {
    let mut items_added = vec![0; requirement_amounts.len()];
    let mut can_progress = *items_total <= item_capacity && accum.iter().all(|value| *value < 1.0);

    if can_progress {
        let shift = edelta * deconstruct_speed / build_time;
        let real_shift = shift.min(1.0 - *progress);
        *progress += shift;
        *time += edelta;

        for (accum, amount) in accum.iter_mut().zip(requirement_amounts.iter()) {
            *accum += *amount as f32 * cost_multiplier * real_shift;
        }
    }

    for (index, value) in accum.iter_mut().enumerate() {
        let free = item_capacity - *items_total;
        if free <= 0 {
            break;
        }
        let taken = (*value as i32).min(free);
        if taken > 0 {
            *items_total += taken;
            *value -= taken as f32;
            items_added[index] += taken;
        }
    }

    let mut finished = false;
    if *progress >= 1.0 {
        can_progress = true;
        for (index, value) in accum.iter_mut().enumerate() {
            if (*value - 1.0).abs() <= 0.0001 {
                if *items_total < item_capacity {
                    *items_total += 1;
                    *value = 0.0;
                    items_added[index] += 1;
                } else {
                    can_progress = false;
                    break;
                }
            }
        }
        finished = can_progress;
    }

    PayloadDeconstructorProgressStep {
        can_progress,
        finished,
        items_added,
    }
}

pub fn write_deconstructor_extra<W: Write>(
    write: &mut W,
    progress: f32,
    accum: Option<&[f32]>,
) -> io::Result<()> {
    write_f32(write, progress)?;
    let len = accum.map_or(0, |values| values.len() as i16);
    write_i16(write, len)?;
    if let Some(values) = accum {
        for value in values {
            write_f32(write, *value)?;
        }
    }
    Ok(())
}

pub fn read_deconstructor_extra<R: Read>(read: &mut R) -> io::Result<(f32, Option<Vec<f32>>)> {
    let progress = read_f32(read)?;
    let len = read_i16(read)?;
    let accum = if len > 0 {
        let mut values = Vec::with_capacity(len as usize);
        for _ in 0..len {
            values.push(read_f32(read)?);
        }
        Some(values)
    } else {
        None
    };
    Ok((progress, accum))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PayloadLoaderState {
    pub has_payload: bool,
    pub exporting: bool,
    pub payload_has_items: bool,
    pub payload_items_total: i32,
    pub payload_item_capacity: i32,
    pub payload_has_liquids: bool,
    pub loader_liquid_amount: f32,
    pub payload_liquid_amount: f32,
    pub payload_liquid_capacity: f32,
    pub has_battery: bool,
    pub payload_power_status: f32,
}

impl Default for PayloadLoaderState {
    fn default() -> Self {
        Self {
            has_payload: false,
            exporting: false,
            payload_has_items: false,
            payload_items_total: 0,
            payload_item_capacity: 0,
            payload_has_liquids: false,
            loader_liquid_amount: 0.0,
            payload_liquid_amount: 0.0,
            payload_liquid_capacity: 0.0,
            has_battery: false,
            payload_power_status: 0.0,
        }
    }
}

pub fn payload_loader_should_export(state: &PayloadLoaderState) -> bool {
    state.has_payload
        && (state.exporting
            || (state.payload_has_liquids
                && state.loader_liquid_amount >= 0.1
                && state.payload_liquid_amount >= state.payload_liquid_capacity - 0.001)
            || (state.has_battery && state.payload_power_status >= 0.999_999_999))
}

pub fn payload_loader_accept_payload(
    base_accepts: bool,
    payload_fits: bool,
    is_build_payload: bool,
    block_has_items: bool,
    unloadable: bool,
    item_capacity: i32,
    block_size: i32,
    max_block_size: i32,
    block_has_liquids: bool,
    liquid_capacity: f32,
    has_buffered_power: bool,
) -> bool {
    base_accepts
        && payload_fits
        && is_build_payload
        && ((block_has_items && unloadable && item_capacity >= 10 && block_size <= max_block_size)
            || (block_has_liquids && liquid_capacity >= 10.0)
            || has_buffered_power)
}

pub fn payload_loader_accept_item(
    items_total: i32,
    item_capacity: i32,
    source_is_payload_unloader: bool,
) -> bool {
    items_total < item_capacity && !source_is_payload_unloader
}

pub fn payload_loader_accept_liquid(
    current_liquid_matches: bool,
    current_amount: f32,
    source_is_payload_unloader: bool,
) -> bool {
    (current_liquid_matches || current_amount < 0.2) && !source_is_payload_unloader
}

pub fn payload_loader_liquid_flow(
    liquids_loaded: f32,
    edelta: f32,
    payload_liquid_capacity: f32,
    payload_liquid_amount: f32,
    loader_liquid_amount: f32,
) -> f32 {
    (liquids_loaded * edelta)
        .min(payload_liquid_capacity - payload_liquid_amount)
        .min(loader_liquid_amount)
        .max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PayloadLoaderPowerStep {
    pub payload_power_status: f32,
    pub exporting: bool,
    pub available_input: f32,
}

pub fn payload_loader_charge_battery(
    payload_power_status: f32,
    loader_power_status: f32,
    base_power_use: f32,
    max_power_consumption: f32,
    battery_capacity: f32,
    edelta: f32,
) -> PayloadLoaderPowerStep {
    let power_input = loader_power_status * (base_power_use + max_power_consumption);
    let available_input = (power_input - base_power_use).max(0.0);
    let mut next = payload_power_status + available_input / battery_capacity * edelta;
    let exporting = next >= 1.0;
    if exporting {
        next = next.clamp(0.0, 1.0);
    }
    PayloadLoaderPowerStep {
        payload_power_status: next,
        exporting,
        available_input,
    }
}

pub fn payload_unloader_should_export(state: &PayloadLoaderState) -> bool {
    state.has_payload
        && (!state.payload_has_items || state.payload_items_total == 0)
        && (!state.payload_has_liquids || state.payload_liquid_amount <= 0.011)
        && (!state.has_battery || state.payload_power_status <= 0.000_000_1)
}

pub fn payload_unloader_accept_item() -> bool {
    false
}

pub fn payload_unloader_accept_liquid() -> bool {
    false
}

pub fn payload_unloader_full(items_total: i32, item_capacity: i32) -> bool {
    items_total >= item_capacity
}

pub fn payload_unloader_liquid_flow(
    liquids_loaded: f32,
    edelta: f32,
    unloader_liquid_capacity: f32,
    unloader_liquid_amount: f32,
    payload_liquid_amount: f32,
) -> f32 {
    (liquids_loaded * edelta)
        .min(unloader_liquid_capacity - unloader_liquid_amount)
        .min(payload_liquid_amount)
        .max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PayloadUnloaderPowerStep {
    pub payload_power_status: f32,
    pub last_output_power: f32,
}

pub fn payload_unloader_drain_battery(
    payload_power_status: f32,
    battery_capacity: f32,
    max_power_unload: f32,
    edelta: f32,
) -> PayloadUnloaderPowerStep {
    let total = payload_power_status * battery_capacity;
    let unloaded = (max_power_unload * edelta).min(total).max(0.0);
    PayloadUnloaderPowerStep {
        payload_power_status: payload_power_status - unloaded / battery_capacity,
        last_output_power: unloaded,
    }
}

pub fn write_payload_loader_extra<W: Write>(write: &mut W, exporting: bool) -> io::Result<()> {
    write_bool(write, exporting)
}

pub fn read_payload_loader_extra<R: Read>(read: &mut R, revision: u8) -> io::Result<bool> {
    if revision >= 1 {
        read_bool(read)
    } else {
        Ok(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadDriverState {
    Idle,
    Accepting,
    Shooting,
}

impl PayloadDriverState {
    pub fn ordinal(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Accepting => 1,
            Self::Shooting => 2,
        }
    }

    pub fn from_ordinal(value: u8) -> io::Result<Self> {
        match value {
            0 => Ok(Self::Idle),
            1 => Ok(Self::Accepting),
            2 => Ok(Self::Shooting),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown payload driver state",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PayloadMassDriverState {
    pub link: i32,
    pub turret_rotation: f32,
    pub state: PayloadDriverState,
    pub reload_counter: f32,
    pub charge: f32,
    pub loaded: bool,
    pub charging: bool,
}

impl Default for PayloadMassDriverState {
    fn default() -> Self {
        Self {
            link: -1,
            turret_rotation: 90.0,
            state: PayloadDriverState::Idle,
            reload_counter: 0.0,
            charge: 0.0,
            loaded: false,
            charging: false,
        }
    }
}

pub fn payload_mass_driver_config_from_relative(tile_x: i32, tile_y: i32, dx: i32, dy: i32) -> i32 {
    point2_pack(dx + tile_x, dy + tile_y)
}

pub fn payload_mass_driver_config_relative(link: i32, tile_x: i32, tile_y: i32) -> (i32, i32) {
    (
        point2_x(link) as i32 - tile_x,
        point2_y(link) as i32 - tile_y,
    )
}

pub fn payload_mass_driver_accept_payload(payload_size: f32, max_payload_size: f32) -> bool {
    payload_size <= max_payload_size
}

pub fn payload_mass_driver_progress(reload_counter: f32, reload: f32) -> f32 {
    (1.0 - reload_counter / reload).clamp(0.0, 1.0)
}

pub fn payload_mass_driver_discharge(charge: &mut f32, charging: bool, delta: f32) {
    if !charging {
        *charge = (*charge - delta * 10.0).max(0.0);
    }
}

pub fn payload_mass_driver_reload_tick(reload_counter: &mut f32, edelta: f32, reload: f32) {
    *reload_counter = (*reload_counter - edelta / reload).max(0.0);
}

pub fn payload_mass_driver_idle_next(
    waiting_shooters_empty: bool,
    has_payload: bool,
    has_link: bool,
) -> PayloadDriverState {
    if !waiting_shooters_empty && !has_payload {
        PayloadDriverState::Accepting
    } else if has_link {
        PayloadDriverState::Shooting
    } else {
        PayloadDriverState::Idle
    }
}

pub fn payload_mass_driver_accepting_should_idle(
    current_shooter_exists: bool,
    has_payload: bool,
) -> bool {
    !current_shooter_exists || has_payload
}

pub fn payload_mass_driver_shooting_should_idle(
    has_link: bool,
    waiting_shooters_empty: bool,
    has_payload: bool,
) -> bool {
    !has_link || (!waiting_shooters_empty && !has_payload)
}

pub fn payload_mass_driver_loaded_pay_length(
    length: f32,
    reload_counter: f32,
    knockback: f32,
) -> f32 {
    length - reload_counter * knockback
}

pub fn payload_mass_driver_ready_to_fire(
    moved_out: bool,
    has_payload: bool,
    other_payload_empty: bool,
    reload_counter: f32,
    other_current_is_self: bool,
    other_state: PayloadDriverState,
    other_reload_counter: f32,
    turret_rotation: f32,
    target_rotation: f32,
    other_turret_rotation: f32,
) -> bool {
    moved_out
        && has_payload
        && other_payload_empty
        && reload_counter <= 0.0
        && other_current_is_self
        && other_state == PayloadDriverState::Accepting
        && other_reload_counter <= 0.0
        && angle_within(turret_rotation, target_rotation, 1.0)
        && angle_within(other_turret_rotation, target_rotation + 180.0, 1.0)
}

pub fn payload_mass_driver_charge_until_fire(
    charge: &mut f32,
    edelta: f32,
    charge_time: f32,
    ready_to_fire: bool,
) -> bool {
    if ready_to_fire {
        *charge += edelta;
        *charge >= charge_time
    } else {
        false
    }
}

pub fn payload_mass_driver_reset_after_fire(state: &mut PayloadMassDriverState) {
    state.charge = 0.0;
    state.loaded = false;
    state.charging = false;
    state.state = PayloadDriverState::Idle;
    state.reload_counter = 1.0;
}

pub fn write_payload_mass_driver_extra<W: Write>(
    write: &mut W,
    state: &PayloadMassDriverState,
) -> io::Result<()> {
    write_i32(write, state.link)?;
    write_f32(write, state.turret_rotation)?;
    write_u8(write, state.state.ordinal())?;
    write_f32(write, state.reload_counter)?;
    write_f32(write, state.charge)?;
    write_bool(write, state.loaded)?;
    write_bool(write, state.charging)
}

pub fn read_payload_mass_driver_extra<R: Read>(
    read: &mut R,
    revision: u8,
) -> io::Result<PayloadMassDriverState> {
    let link = read_i32(read)?;
    let turret_rotation = read_f32(read)?;
    let state = PayloadDriverState::from_ordinal(read_u8(read)?)?;
    let mut value = PayloadMassDriverState {
        link,
        turret_rotation,
        state,
        ..Default::default()
    };
    if revision >= 1 {
        value.reload_counter = read_f32(read)?;
        value.charge = read_f32(read)?;
        value.loaded = read_bool(read)?;
        value.charging = read_bool(read)?;
    }
    Ok(value)
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn move_toward_angle(from: f32, to: f32, amount: f32) -> f32 {
    let delta = angle_delta(from, to);
    if delta.abs() <= amount {
        normalize_angle(to)
    } else {
        normalize_angle(from + delta.signum() * amount)
    }
}

fn angle_delta(from: f32, to: f32) -> f32 {
    let mut delta = (to - from) % 360.0;
    if delta > 180.0 {
        delta -= 360.0;
    } else if delta < -180.0 {
        delta += 360.0;
    }
    delta
}

fn normalize_angle(value: f32) -> f32 {
    let mut value = value % 360.0;
    if value < 0.0 {
        value += 360.0;
    }
    value
}

fn angle_within(a: f32, b: f32, margin: f32) -> bool {
    angle_delta(a, b).abs() <= margin
}

fn write_vec_nullable<W: Write>(write: &mut W, value: Option<Vec2>) -> io::Result<()> {
    match value {
        Some(value) => {
            write_f32(write, value.x)?;
            write_f32(write, value.y)
        }
        None => {
            write_f32(write, f32::NAN)?;
            write_f32(write, f32::NAN)
        }
    }
}

fn read_vec_nullable<R: Read>(read: &mut R) -> io::Result<Option<Vec2>> {
    let x = read_f32(read)?;
    let y = read_f32(read)?;
    Ok((!x.is_nan() && !y.is_nan()).then_some(Vec2 { x, y }))
}

fn read_optional_i16<R: Read>(read: &mut R) -> io::Result<Option<i16>> {
    let id = read_i16(read)?;
    Ok((id >= 0).then_some(id))
}

fn read_bool<R: Read>(read: &mut R) -> io::Result<bool> {
    Ok(read_u8(read)? != 0)
}

fn write_bool<W: Write>(write: &mut W, value: bool) -> io::Result<()> {
    write_u8(write, u8::from(value))
}

fn read_u8<R: Read>(read: &mut R) -> io::Result<u8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn write_u8<W: Write>(write: &mut W, value: u8) -> io::Result<()> {
    write.write_all(&[value])
}

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_i32<R: Read>(read: &mut R) -> io::Result<i32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

fn write_i32<W: Write>(write: &mut W, value: i32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

fn read_f32<R: Read>(read: &mut R) -> io::Result<f32> {
    let mut buf = [0; 4];
    read.read_exact(&mut buf)?;
    Ok(f32::from_be_bytes(buf))
}

fn write_f32<W: Write>(write: &mut W, value: f32) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_ref_presence_and_headers_match_java_payload_write() {
        let mut bytes = Vec::new();
        write_payload_ref(&mut bytes, None).unwrap();
        assert_eq!(bytes, vec![0]);

        let block = PayloadRef::Block {
            block: 12,
            version: 3,
            build_bytes: vec![0xaa, 0xbb],
        };
        let mut bytes = Vec::new();
        write_payload_ref(&mut bytes, Some(&block)).unwrap();
        assert_eq!(bytes, vec![1, PAYLOAD_BLOCK_TYPE, 0, 12, 3, 0xaa, 0xbb]);

        let unit = PayloadRef::Unit {
            class_id: 9,
            unit_bytes: vec![1, 2],
        };
        let mut bytes = Vec::new();
        write_payload_ref(&mut bytes, Some(&unit)).unwrap();
        assert_eq!(bytes, vec![1, PAYLOAD_UNIT_TYPE, 9, 1, 2]);
    }

    #[test]
    fn payload_block_common_state_moves_and_serializes_empty_payload_prefix() {
        let payload = PayloadRef::Unit {
            class_id: 7,
            unit_bytes: vec![0xde, 0xad],
        };
        let mut state = PayloadBlockBuildState::default();
        assert!(payload_block_accept_payload(&state));
        payload_block_handle_payload(
            &mut state,
            payload,
            Vec2 { x: 10.0, y: 20.0 },
            Vec2 {
                x: 100.0,
                y: -100.0,
            },
            270.0,
            3,
            8.0,
        );
        assert!(!payload_block_accept_payload(&state));
        assert_eq!(state.pay_vector, Vec2 { x: 12.0, y: -12.0 });
        assert_eq!(state.pay_rotation, 270.0);

        assert!(!payload_block_move_in(
            &mut state, true, true, 90.0, 6.0, 90.0, 1.0
        ));
        assert_eq!(state.pay_rotation, 180.0);
        assert!(state.pay_vector.len() < 17.0);

        state.pay_vector = Vec2::ZERO;
        assert!(payload_block_move_in(
            &mut state, false, true, 0.0, 6.0, 90.0, 1.0
        ));
        assert!(payload_block_take_payload(&mut state).is_some());
        assert!(payload_block_accept_payload(&state));

        payload_block_picked_up(&mut state);
        assert!(state.carried);
        payload_block_draw_team_top(&mut state);
        assert!(!state.carried);

        state.pay_vector = Vec2 { x: 1.0, y: 2.0 };
        state.pay_rotation = 45.0;
        let mut bytes = Vec::new();
        write_payload_block_build_common(&mut bytes, &state).unwrap();
        let restored = read_empty_payload_block_build_common(&mut bytes.as_slice()).unwrap();
        assert_eq!(restored.pay_vector, Vec2 { x: 1.0, y: 2.0 });
        assert_eq!(restored.pay_rotation, 45.0);
        assert_eq!(restored.payload, None);
    }

    #[test]
    fn payload_block_move_out_uses_rotation_target_and_arrival_threshold() {
        let mut state = PayloadBlockBuildState {
            payload: Some(PayloadRef::Block {
                block: 1,
                version: 0,
                build_bytes: vec![],
            }),
            pay_vector: Vec2::ZERO,
            pay_rotation: 180.0,
            carried: false,
        };
        let dest = payload_block_move_out_target(0.0, 3, 8.0);
        assert_eq!(dest, Vec2 { x: 12.0, y: 0.0 });

        assert!(!payload_block_move_out_step(
            &mut state, 0.0, 3, 8.0, 6.0, 90.0, 1.0
        ));
        assert_eq!(state.pay_vector, Vec2 { x: 6.0, y: 0.0 });
        assert_eq!(state.pay_rotation, 90.0);

        assert!(payload_block_move_out_step(
            &mut state, 0.0, 3, 8.0, 6.0, 90.0, 1.0
        ));
        assert_eq!(state.pay_vector, dest);
        assert_eq!(state.pay_rotation, 0.0);
    }

    #[test]
    fn payload_source_configuration_clears_opposite_target_and_serializes_extra() {
        let mut state = PayloadSourceState::default();
        payload_source_configure_block(&mut state, 5);
        assert_eq!(
            payload_source_update(&mut state),
            PayloadSourceSpawn::Block(5)
        );
        assert!(state.has_payload);

        payload_source_configure_unit(&mut state, 8);
        assert_eq!(state.config_block, None);
        assert!(!state.has_payload);
        assert_eq!(
            payload_source_update(&mut state),
            PayloadSourceSpawn::Unit(8)
        );
        assert!(!payload_source_accept_payload());

        let mut bytes = Vec::new();
        write_payload_source_extra(&mut bytes, Some(8), None, Some(Vec2 { x: 1.5, y: -2.0 }))
            .unwrap();
        let (unit, block, command) = read_payload_source_extra(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(unit, Some(8));
        assert_eq!(block, None);
        assert_eq!(command, Some(Vec2 { x: 1.5, y: -2.0 }));

        let mut bytes = Vec::new();
        write_payload_source_extra(&mut bytes, None, Some(5), None).unwrap();
        assert!(bytes[4..12].iter().any(|byte| *byte != 0));
        let (_, block, command) = read_payload_source_extra(&mut bytes.as_slice(), 1).unwrap();
        assert_eq!(block, Some(5));
        assert_eq!(command, None);
    }

    #[test]
    fn constructor_recipe_resets_progress_and_roundtrips_short() {
        let mut recipe = Some(1);
        let mut progress = 0.75;
        constructor_configure(&mut recipe, &mut progress, 2, true);
        assert_eq!(recipe, Some(2));
        assert_eq!(progress, 0.0);
        constructor_clear(&mut recipe);
        assert_eq!(recipe, None);

        let mut bytes = Vec::new();
        write_constructor_recipe(&mut bytes, Some(2)).unwrap();
        assert_eq!(
            read_constructor_recipe(&mut bytes.as_slice()).unwrap(),
            Some(2)
        );
    }

    #[test]
    fn deconstructor_accept_begin_and_accum_serialization_follow_java_order() {
        let empty = PayloadDeconstructorState::default();
        assert!(payload_deconstructor_accept_payload(&empty, 3, 4.0, 4.0));
        assert!(payload_deconstructor_should_consume(true, true));
        assert!(!payload_deconstructor_should_consume(true, false));

        let mut state = PayloadDeconstructorState {
            has_payload: true,
            ..Default::default()
        };
        assert!(payload_deconstructor_begin_if_arrived(&mut state, true, 3));
        assert!(state.has_deconstructing);
        assert!(!state.has_payload);
        assert_eq!(state.accum, Some(vec![0.0; 3]));

        let mut bytes = Vec::new();
        write_deconstructor_extra(&mut bytes, 0.5, Some(&[1.0, 2.0])).unwrap();
        let (progress, accum) = read_deconstructor_extra(&mut bytes.as_slice()).unwrap();
        assert_eq!(progress, 0.5);
        assert_eq!(accum, Some(vec![1.0, 2.0]));
    }

    #[test]
    fn deconstructor_progress_accumulates_outputs_and_finishes_like_java() {
        let mut progress = 0.0;
        let mut time = 0.0;
        let mut accum = vec![0.0, 0.0];
        let mut total = 0;
        let step = payload_deconstructor_update_progress(
            &mut progress,
            &mut time,
            &mut accum,
            &[4, 2],
            &mut total,
            100,
            50.0,
            2.0,
            100.0,
            1.0,
        );
        assert_eq!(
            step,
            PayloadDeconstructorProgressStep {
                can_progress: true,
                finished: true,
                items_added: vec![4, 2]
            }
        );
        assert_eq!(progress, 1.0);
        assert_eq!(time, 50.0);
        assert_eq!(accum, vec![0.0, 0.0]);
        assert_eq!(total, 6);

        let mut progress = 0.25;
        let mut time = 0.0;
        let mut accum = vec![1.0];
        let mut total = 10;
        let step = payload_deconstructor_update_progress(
            &mut progress,
            &mut time,
            &mut accum,
            &[4],
            &mut total,
            100,
            10.0,
            2.0,
            100.0,
            1.0,
        );
        assert!(!step.can_progress);
        assert_eq!(progress, 0.25);
        assert_eq!(time, 0.0);
        assert_eq!(step.items_added, vec![1]);
        assert_eq!(accum, vec![0.0]);
        assert_eq!(total, 11);

        let mut progress = 1.0;
        let mut time = 0.0;
        let mut accum = vec![1.0];
        let mut total = 100;
        let step = payload_deconstructor_update_progress(
            &mut progress,
            &mut time,
            &mut accum,
            &[1],
            &mut total,
            100,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        assert!(!step.finished);
        assert!(!step.can_progress);
        assert_eq!(accum, vec![1.0]);
    }

    #[test]
    fn loader_unloader_export_and_loader_revision_flag_match_upstream() {
        assert!(payload_loader_accept_payload(
            true, true, true, true, true, 10, 3, 3, false, 0.0, false
        ));
        assert!(payload_loader_accept_payload(
            true, true, true, false, false, 0, 5, 3, true, 10.0, false
        ));
        assert!(payload_loader_accept_payload(
            true, true, true, false, false, 0, 5, 3, false, 0.0, true
        ));
        assert!(!payload_loader_accept_payload(
            true, true, false, true, true, 100, 3, 3, false, 0.0, false
        ));
        assert!(payload_loader_accept_item(99, 100, false));
        assert!(!payload_loader_accept_item(100, 100, false));
        assert!(!payload_loader_accept_item(0, 100, true));
        assert!(payload_loader_accept_liquid(true, 50.0, false));
        assert!(payload_loader_accept_liquid(false, 0.1, false));
        assert!(!payload_loader_accept_liquid(false, 0.2, false));
        assert!(!payload_loader_accept_liquid(true, 0.0, true));

        let loader = PayloadLoaderState {
            has_payload: true,
            payload_has_liquids: true,
            loader_liquid_amount: 0.2,
            payload_liquid_amount: 99.999,
            payload_liquid_capacity: 100.0,
            ..Default::default()
        };
        assert!(payload_loader_should_export(&loader));
        assert_eq!(
            payload_loader_liquid_flow(40.0, 0.5, 100.0, 90.0, 50.0),
            10.0
        );
        assert_eq!(
            payload_loader_liquid_flow(40.0, 0.5, 100.0, 10.0, 12.0),
            12.0
        );
        assert_eq!(
            payload_loader_charge_battery(0.5, 1.0, 2.0, 40.0, 100.0, 1.0),
            PayloadLoaderPowerStep {
                payload_power_status: 0.9,
                exporting: false,
                available_input: 40.0
            }
        );
        assert_eq!(
            payload_loader_charge_battery(0.9, 1.0, 2.0, 40.0, 100.0, 1.0),
            PayloadLoaderPowerStep {
                payload_power_status: 1.0,
                exporting: true,
                available_input: 40.0
            }
        );

        let unloader = PayloadLoaderState {
            has_payload: true,
            payload_has_items: true,
            payload_items_total: 0,
            payload_has_liquids: true,
            payload_liquid_amount: 0.01,
            has_battery: true,
            payload_power_status: 0.0,
            ..Default::default()
        };
        assert!(payload_unloader_should_export(&unloader));
        assert!(!payload_unloader_accept_item());
        assert!(!payload_unloader_accept_liquid());
        assert!(payload_unloader_full(100, 100));
        assert!(!payload_unloader_full(99, 100));
        assert_eq!(
            payload_unloader_liquid_flow(40.0, 0.5, 100.0, 95.0, 50.0),
            5.0
        );
        assert_eq!(
            payload_unloader_drain_battery(0.5, 100.0, 80.0, 0.25),
            PayloadUnloaderPowerStep {
                payload_power_status: 0.3,
                last_output_power: 20.0
            }
        );

        let mut bytes = Vec::new();
        write_payload_loader_extra(&mut bytes, true).unwrap();
        assert_eq!(
            read_payload_loader_extra(&mut bytes.as_slice(), 1).unwrap(),
            true
        );
        assert_eq!(
            read_payload_loader_extra(&mut [].as_slice(), 0).unwrap(),
            false
        );
    }

    #[test]
    fn payload_void_and_mass_driver_config_and_serialization_match_java_fields() {
        assert!(payload_void_accept_unit_payload());
        assert!(payload_void_update(true, 1.0, true));
        assert!(!payload_void_update(true, 0.0, true));

        let packed = payload_mass_driver_config_from_relative(10, 20, -2, 3);
        assert_eq!(payload_mass_driver_config_relative(packed, 10, 20), (-2, 3));
        assert!(payload_mass_driver_accept_payload(24.0, 24.0));
        assert_eq!(payload_mass_driver_progress(15.0, 30.0), 0.5);

        let state = PayloadMassDriverState {
            link: packed,
            turret_rotation: 45.0,
            state: PayloadDriverState::Shooting,
            reload_counter: 0.25,
            charge: 10.0,
            loaded: true,
            charging: true,
        };
        let mut bytes = Vec::new();
        write_payload_mass_driver_extra(&mut bytes, &state).unwrap();
        assert_eq!(
            read_payload_mass_driver_extra(&mut bytes.as_slice(), 1).unwrap(),
            state
        );
    }

    #[test]
    fn payload_mass_driver_state_machine_helpers_follow_java_conditions() {
        assert_eq!(
            payload_mass_driver_idle_next(false, false, true),
            PayloadDriverState::Accepting
        );
        assert_eq!(
            payload_mass_driver_idle_next(true, false, true),
            PayloadDriverState::Shooting
        );
        assert_eq!(
            payload_mass_driver_idle_next(true, false, false),
            PayloadDriverState::Idle
        );
        assert!(payload_mass_driver_accepting_should_idle(false, false));
        assert!(payload_mass_driver_accepting_should_idle(true, true));
        assert!(!payload_mass_driver_accepting_should_idle(true, false));
        assert!(payload_mass_driver_shooting_should_idle(false, true, true));
        assert!(payload_mass_driver_shooting_should_idle(true, false, false));
        assert!(!payload_mass_driver_shooting_should_idle(true, true, false));

        let mut charge = 25.0;
        payload_mass_driver_discharge(&mut charge, false, 1.0);
        assert_eq!(charge, 15.0);
        payload_mass_driver_discharge(&mut charge, false, 2.0);
        assert_eq!(charge, 0.0);
        let mut reload_counter = 0.5;
        payload_mass_driver_reload_tick(&mut reload_counter, 15.0, 30.0);
        assert_eq!(reload_counter, 0.0);
        assert_eq!(
            payload_mass_driver_loaded_pay_length(11.125, 0.5, 5.0),
            8.625
        );

        assert!(payload_mass_driver_ready_to_fire(
            true,
            true,
            true,
            0.0,
            true,
            PayloadDriverState::Accepting,
            0.0,
            44.5,
            45.0,
            225.5,
        ));
        assert!(!payload_mass_driver_ready_to_fire(
            true,
            true,
            true,
            0.0,
            true,
            PayloadDriverState::Accepting,
            0.0,
            43.0,
            45.0,
            225.0,
        ));

        let mut charge = 99.0;
        assert!(payload_mass_driver_charge_until_fire(
            &mut charge,
            1.0,
            100.0,
            true
        ));
        let mut state = PayloadMassDriverState {
            charge,
            loaded: true,
            charging: true,
            state: PayloadDriverState::Shooting,
            reload_counter: 0.0,
            ..Default::default()
        };
        payload_mass_driver_reset_after_fire(&mut state);
        assert_eq!(state.charge, 0.0);
        assert!(!state.loaded);
        assert!(!state.charging);
        assert_eq!(state.state, PayloadDriverState::Idle);
        assert_eq!(state.reload_counter, 1.0);
    }
}
