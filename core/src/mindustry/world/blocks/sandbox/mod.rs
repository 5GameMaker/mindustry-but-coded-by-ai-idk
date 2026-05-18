use std::io::{self, Read, Write};

use crate::mindustry::ctype::ContentId;

pub const DEFAULT_ITEM_SOURCE_ITEMS_PER_SECOND: i32 = 100;
pub const DEFAULT_LIQUID_SOURCE_CAPACITY: f32 = 10_000.0;
pub const DEFAULT_POWER_SOURCE_PRODUCTION: f32 = 10_000.0;
pub const POWER_VOID_CONSUMPTION: f32 = f32::MAX;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerSourceState {
    pub enabled: bool,
}

impl Default for PowerSourceState {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub fn power_source_on_proximity_update(state: &mut PowerSourceState, allow_update: bool) {
    if !allow_update {
        state.enabled = false;
    }
}

pub fn power_source_production(enabled: bool, power_production: f32) -> f32 {
    if enabled {
        power_production
    } else {
        0.0
    }
}

pub fn power_void_consumption() -> f32 {
    POWER_VOID_CONSUMPTION
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemSourceState {
    pub counter: f32,
    pub output_item: Option<ContentId>,
}

impl Default for ItemSourceState {
    fn default() -> Self {
        Self {
            counter: 0.0,
            output_item: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemSourceStep {
    pub item: Option<ContentId>,
    pub produced: i32,
}

pub fn item_source_configure(state: &mut ItemSourceState, item: Option<ContentId>) {
    state.output_item = item;
}

pub fn item_source_accept_item() -> bool {
    false
}

pub fn item_source_update(
    state: &mut ItemSourceState,
    items_per_second: i32,
    edelta: f32,
) -> ItemSourceStep {
    let Some(item) = state.output_item else {
        return ItemSourceStep {
            item: None,
            produced: 0,
        };
    };

    state.counter += edelta;
    let limit = 60.0 / items_per_second as f32;
    let mut produced = 0;
    while state.counter >= limit {
        produced += 1;
        state.counter -= limit;
    }

    ItemSourceStep {
        item: Some(item),
        produced,
    }
}

pub fn write_item_source_config<W: Write>(
    write: &mut W,
    output_item: Option<ContentId>,
) -> io::Result<()> {
    write_i16(write, output_item.unwrap_or(-1))
}

pub fn read_item_source_config<R: Read>(read: &mut R) -> io::Result<Option<ContentId>> {
    read_optional_i16(read)
}

pub fn item_void_accept_item(enabled: bool) -> bool {
    enabled
}

pub fn item_void_handle_item(enabled: bool, amount: i32) -> i32 {
    if enabled {
        amount
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidSourceState {
    pub source: Option<ContentId>,
    pub stored_liquid: Option<ContentId>,
    pub amount: f32,
}

impl Default for LiquidSourceState {
    fn default() -> Self {
        Self {
            source: None,
            stored_liquid: None,
            amount: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidSourceStep {
    pub dumped: Option<ContentId>,
    pub amount: f32,
}

pub fn liquid_source_configure(state: &mut LiquidSourceState, liquid: Option<ContentId>) {
    state.source = liquid;
}

pub fn liquid_source_update(state: &mut LiquidSourceState, capacity: f32) -> LiquidSourceStep {
    let Some(liquid) = state.source else {
        state.stored_liquid = None;
        state.amount = 0.0;
        return LiquidSourceStep {
            dumped: None,
            amount: 0.0,
        };
    };

    state.stored_liquid = Some(liquid);
    state.amount = capacity;
    LiquidSourceStep {
        dumped: Some(liquid),
        amount: capacity,
    }
}

pub fn write_liquid_source_config<W: Write>(
    write: &mut W,
    source: Option<ContentId>,
) -> io::Result<()> {
    write_i16(write, source.unwrap_or(-1))
}

pub fn read_liquid_source_config<R: Read>(
    read: &mut R,
    revision: u8,
) -> io::Result<Option<ContentId>> {
    if revision == 1 {
        read_optional_i16(read)
    } else {
        let id = read_i8(read)? as i16;
        Ok((id >= 0).then_some(id))
    }
}

pub fn liquid_void_accept_liquid(enabled: bool) -> bool {
    enabled
}

pub fn liquid_void_handle_liquid(enabled: bool, amount: f32) -> f32 {
    if enabled {
        amount
    } else {
        0.0
    }
}

fn read_optional_i16<R: Read>(read: &mut R) -> io::Result<Option<ContentId>> {
    let id = read_i16(read)?;
    Ok((id >= 0).then_some(id))
}

fn read_i8<R: Read>(read: &mut R) -> io::Result<i8> {
    let mut buf = [0; 1];
    read.read_exact(&mut buf)?;
    Ok(i8::from_be_bytes(buf))
}

fn read_i16<R: Read>(read: &mut R) -> io::Result<i16> {
    let mut buf = [0; 2];
    read.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

fn write_i16<W: Write>(write: &mut W, value: i16) -> io::Result<()> {
    write.write_all(&value.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_source_disables_when_update_not_allowed_and_reports_production() {
        let mut state = PowerSourceState::default();
        assert_eq!(
            power_source_production(state.enabled, DEFAULT_POWER_SOURCE_PRODUCTION),
            DEFAULT_POWER_SOURCE_PRODUCTION
        );

        power_source_on_proximity_update(&mut state, false);
        assert!(!state.enabled);
        assert_eq!(
            power_source_production(state.enabled, DEFAULT_POWER_SOURCE_PRODUCTION),
            0.0
        );
        assert_eq!(power_void_consumption(), f32::MAX);
    }

    #[test]
    fn item_source_updates_counter_and_serializes_config_like_java_short() {
        let mut state = ItemSourceState::default();
        item_source_configure(&mut state, Some(4));

        let first = item_source_update(&mut state, DEFAULT_ITEM_SOURCE_ITEMS_PER_SECOND, 0.30);
        assert_eq!(
            first,
            ItemSourceStep {
                item: Some(4),
                produced: 0
            }
        );
        let second = item_source_update(&mut state, DEFAULT_ITEM_SOURCE_ITEMS_PER_SECOND, 0.91);
        assert_eq!(second.produced, 2);
        assert!(state.counter < 0.01);
        assert!(!item_source_accept_item());

        let mut bytes = Vec::new();
        write_item_source_config(&mut bytes, Some(4)).unwrap();
        assert_eq!(bytes, vec![0, 4]);
        assert_eq!(
            read_item_source_config(&mut bytes.as_slice()).unwrap(),
            Some(4)
        );

        let mut bytes = Vec::new();
        write_item_source_config(&mut bytes, None).unwrap();
        assert_eq!(bytes, vec![0xff, 0xff]);
        assert_eq!(
            read_item_source_config(&mut bytes.as_slice()).unwrap(),
            None
        );
    }

    #[test]
    fn item_void_only_accepts_and_counts_flow_when_enabled() {
        assert!(item_void_accept_item(true));
        assert!(!item_void_accept_item(false));
        assert_eq!(item_void_handle_item(true, 1), 1);
        assert_eq!(item_void_handle_item(false, 1), 0);
    }

    #[test]
    fn liquid_source_sets_capacity_or_clears_and_reads_old_revision_byte() {
        let mut state = LiquidSourceState::default();
        liquid_source_configure(&mut state, Some(7));
        let step = liquid_source_update(&mut state, DEFAULT_LIQUID_SOURCE_CAPACITY);
        assert_eq!(step.dumped, Some(7));
        assert_eq!(state.stored_liquid, Some(7));
        assert_eq!(state.amount, DEFAULT_LIQUID_SOURCE_CAPACITY);

        liquid_source_configure(&mut state, None);
        let step = liquid_source_update(&mut state, DEFAULT_LIQUID_SOURCE_CAPACITY);
        assert_eq!(step.dumped, None);
        assert_eq!(state.stored_liquid, None);
        assert_eq!(state.amount, 0.0);

        let mut bytes = Vec::new();
        write_liquid_source_config(&mut bytes, Some(7)).unwrap();
        assert_eq!(
            read_liquid_source_config(&mut bytes.as_slice(), 1).unwrap(),
            Some(7)
        );
        assert_eq!(
            read_liquid_source_config(&mut [7u8].as_slice(), 0).unwrap(),
            Some(7)
        );
        assert_eq!(
            read_liquid_source_config(&mut [0xffu8].as_slice(), 0).unwrap(),
            None
        );
    }

    #[test]
    fn liquid_void_accepts_and_records_flow_only_when_enabled() {
        assert!(liquid_void_accept_liquid(true));
        assert!(!liquid_void_accept_liquid(false));
        assert_eq!(liquid_void_handle_liquid(true, 3.5), 3.5);
        assert_eq!(liquid_void_handle_liquid(false, 3.5), 0.0);
    }
}
