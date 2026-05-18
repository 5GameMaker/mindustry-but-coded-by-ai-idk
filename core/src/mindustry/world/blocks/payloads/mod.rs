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

pub fn payload_unloader_should_export(state: &PayloadLoaderState) -> bool {
    state.has_payload
        && (!state.payload_has_items || state.payload_items_total == 0)
        && (!state.payload_has_liquids || state.payload_liquid_amount <= 0.011)
        && (!state.has_battery || state.payload_power_status <= 0.000_000_1)
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
    fn loader_unloader_export_and_loader_revision_flag_match_upstream() {
        let loader = PayloadLoaderState {
            has_payload: true,
            payload_has_liquids: true,
            loader_liquid_amount: 0.2,
            payload_liquid_amount: 99.999,
            payload_liquid_capacity: 100.0,
            ..Default::default()
        };
        assert!(payload_loader_should_export(&loader));

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
}
