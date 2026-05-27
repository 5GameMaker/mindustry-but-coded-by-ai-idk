use std::collections::HashMap;

use crate::mindustry::entities::comp::{PuddleComp, PuddleLiquid, PuddleTile};
use crate::mindustry::r#type::{CellLiquid, Liquid};
use crate::mindustry::vars::TILE_SIZE;

pub const MAX_LIQUID: f32 = PuddleComp::MAX_LIQUID;

#[derive(Debug, Clone, PartialEq)]
pub struct PuddleLiquidInfo {
    pub name: String,
    pub flammability: f32,
    pub viscosity: f32,
    pub move_through_blocks: bool,
    pub cap_puddles: bool,
    pub temperature: f32,
    pub particle_spacing: f32,
    pub has_particle_effect: bool,
    pub boil_point: f32,
    pub color_rgba: u32,
    pub gas_color_rgba: u32,
    pub vapor_effect: String,
    pub effect: Option<String>,
    pub can_stay_on: Vec<String>,
    /// Mirrors `CellLiquid.react`: if this liquid reacts with the named
    /// incoming liquid, the incoming amount is added to the existing puddle.
    pub reaction_target: Option<String>,
}

impl PuddleLiquidInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            flammability: 0.0,
            viscosity: 0.5,
            move_through_blocks: false,
            cap_puddles: true,
            temperature: 0.5,
            particle_spacing: 60.0,
            has_particle_effect: false,
            boil_point: 2.0,
            color_rgba: 0xffffffff,
            gas_color_rgba: 0xffffffff,
            vapor_effect: "vapor".to_string(),
            effect: Some("none".to_string()),
            can_stay_on: Vec::new(),
            reaction_target: None,
        }
    }

    pub fn same_liquid(&self, other: &Self) -> bool {
        self.name == other.name
    }

    pub fn will_boil(&self, heat_env: f32) -> bool {
        heat_env >= self.boil_point
    }

    pub fn can_stay_on(&self, other: &Self) -> bool {
        self.can_stay_on.iter().any(|name| name == &other.name)
    }

    pub fn react_added(&self, other: &Self, amount: f32) -> f32 {
        if self.reaction_target.as_deref() == Some(other.name.as_str()) {
            amount
        } else {
            0.0
        }
    }

    pub fn to_component_liquid(&self) -> PuddleLiquid {
        PuddleLiquid {
            flammability: self.flammability,
            viscosity: self.viscosity,
            move_through_blocks: self.move_through_blocks,
            cap_puddles: self.cap_puddles,
            temperature: self.temperature,
            particle_spacing: self.particle_spacing,
            has_particle_effect: self.has_particle_effect,
        }
    }
}

impl From<&Liquid> for PuddleLiquidInfo {
    fn from(liquid: &Liquid) -> Self {
        Self {
            name: liquid.name().to_string(),
            flammability: liquid.flammability,
            viscosity: liquid.viscosity,
            move_through_blocks: liquid.move_through_blocks,
            cap_puddles: liquid.cap_puddles,
            temperature: liquid.temperature,
            particle_spacing: liquid.particle_spacing,
            has_particle_effect: liquid.particle_effect != "none",
            boil_point: liquid.boil_point,
            color_rgba: liquid.color_rgba,
            gas_color_rgba: liquid.gas_color_rgba,
            vapor_effect: liquid.vapor_effect.clone(),
            effect: liquid.effect.clone(),
            can_stay_on: liquid.can_stay_on.clone(),
            reaction_target: None,
        }
    }
}

impl From<&CellLiquid> for PuddleLiquidInfo {
    fn from(liquid: &CellLiquid) -> Self {
        let mut info = Self::from(&liquid.liquid);
        info.reaction_target = liquid.spread_target.clone();
        info
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PuddleTileView {
    pub x: i32,
    pub y: i32,
    pub floor_solid: bool,
    pub floor_is_liquid: bool,
    pub floor_liquid: Option<PuddleLiquidInfo>,
    pub build_present: bool,
    pub team: i32,
}

impl PuddleTileView {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            floor_solid: false,
            floor_is_liquid: false,
            floor_liquid: None,
            build_present: false,
            team: 0,
        }
    }

    pub fn with_liquid_floor(mut self, liquid: PuddleLiquidInfo) -> Self {
        self.floor_is_liquid = true;
        self.floor_liquid = Some(liquid);
        self
    }

    pub fn solid_floor(mut self) -> Self {
        self.floor_solid = true;
        self
    }

    pub fn with_build(mut self, team: i32) -> Self {
        self.build_present = true;
        self.team = team;
        self
    }

    pub fn world_x(&self) -> f32 {
        self.x as f32 * TILE_SIZE as f32
    }

    pub fn world_y(&self) -> f32 {
        self.y as f32 * TILE_SIZE as f32
    }

    pub fn same_tile(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn to_component_tile(&self) -> PuddleTile {
        PuddleTile {
            x: self.x,
            y: self.y,
            build_present: self.build_present,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PuddleDepositContext {
    pub initial: bool,
    pub cap: bool,
    pub net_client: bool,
    pub space: bool,
    pub heat_env: f32,
    pub time: f32,
    pub vapor_chance_passed: bool,
    pub space_liquid_chance_passed: bool,
    pub fireball_chance_passed: bool,
    pub steam_chance_passed: bool,
}

impl Default for PuddleDepositContext {
    fn default() -> Self {
        Self {
            initial: true,
            cap: false,
            net_client: false,
            space: false,
            heat_env: 0.0,
            time: 0.0,
            vapor_chance_passed: false,
            space_liquid_chance_passed: false,
            fireball_chance_passed: false,
            steam_chance_passed: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuddleDepositOutcome {
    Ignored,
    Boiled,
    Space,
    ReactedWithFloor,
    Created,
    Accepted,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PuddleDepositResult {
    pub outcome: PuddleDepositOutcome,
    pub tile: Option<(i32, i32)>,
    pub created: bool,
    pub vapor_effect: bool,
    pub space_liquid: bool,
    pub ripple: bool,
    pub steam: bool,
    pub create_fire: bool,
    pub fireball: bool,
    pub amount: f32,
    pub accepting: f32,
    pub added: f32,
}

impl PuddleDepositResult {
    fn ignored(tile: Option<(i32, i32)>) -> Self {
        Self {
            outcome: PuddleDepositOutcome::Ignored,
            tile,
            created: false,
            vapor_effect: false,
            space_liquid: false,
            ripple: false,
            steam: false,
            create_fire: false,
            fireball: false,
            amount: 0.0,
            accepting: 0.0,
            added: 0.0,
        }
    }

    fn with_outcome(outcome: PuddleDepositOutcome, tile: (i32, i32)) -> Self {
        Self {
            outcome,
            tile: Some(tile),
            ..Self::ignored(Some(tile))
        }
    }

    fn apply_reaction(mut self, reaction: PuddleReactionResult) -> Self {
        self.steam = reaction.steam;
        self.create_fire = reaction.create_fire;
        self.fireball = reaction.fireball;
        self.added = reaction.added;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PuddleReactionResult {
    pub added: f32,
    pub create_fire: bool,
    pub fireball: bool,
    pub steam: bool,
}

impl PuddleReactionResult {
    pub fn none() -> Self {
        Self {
            added: 0.0,
            create_fire: false,
            fireball: false,
            steam: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PuddleEntry {
    pub puddle: PuddleComp,
    pub liquid: PuddleLiquidInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Puddles {
    width: i32,
    height: i32,
    next_id: i32,
    puddles: HashMap<(i32, i32), PuddleEntry>,
}

impl Puddles {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width: width.max(0),
            height: height.max(0),
            next_id: 0,
            puddles: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.puddles.len()
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn is_empty(&self) -> bool {
        self.puddles.is_empty()
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&PuddleComp> {
        self.puddles.get(&(x, y)).map(|entry| &entry.puddle)
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut PuddleComp> {
        self.puddles.get_mut(&(x, y)).map(|entry| &mut entry.puddle)
    }

    pub fn get_entry(&self, x: i32, y: i32) -> Option<&PuddleEntry> {
        self.puddles.get(&(x, y))
    }

    pub fn entries(&self) -> impl Iterator<Item = (&(i32, i32), &PuddleEntry)> {
        self.puddles.iter()
    }

    pub fn slurp_matching_liquid(&mut self, x: i32, y: i32, liquid_name: &str, amount: f32) -> f32 {
        let Some(entry) = self.puddles.get_mut(&(x, y)) else {
            return 0.0;
        };
        if entry.liquid.name != liquid_name {
            return 0.0;
        }
        let taken = entry.puddle.amount.min(amount.max(0.0));
        entry.puddle.amount -= taken;
        taken
    }

    pub fn has_liquid(&self, tile: Option<&PuddleTileView>, liquid: &PuddleLiquidInfo) -> bool {
        let Some(tile) = tile else {
            return false;
        };

        self.puddles
            .get(&(tile.x, tile.y))
            .is_some_and(|entry| entry.liquid.same_liquid(liquid) && entry.puddle.amount >= 0.5)
    }

    pub fn remove(&mut self, tile: Option<&PuddleTileView>) -> Option<PuddleEntry> {
        tile.and_then(|tile| self.puddles.remove(&(tile.x, tile.y)))
    }

    pub fn register(&mut self, mut puddle: PuddleComp, liquid: PuddleLiquidInfo) -> bool {
        let Some(tile) = puddle.tile else {
            return false;
        };

        if !self.in_bounds(tile.x, tile.y) {
            return false;
        }

        puddle.registered = true;
        puddle.liquid = Some(liquid.to_component_liquid());
        self.puddles
            .insert((tile.x, tile.y), PuddleEntry { puddle, liquid });
        true
    }

    pub fn deposit_at(
        &mut self,
        tile: Option<PuddleTileView>,
        liquid: PuddleLiquidInfo,
        amount: f32,
        context: PuddleDepositContext,
    ) -> PuddleDepositResult {
        self.deposit(tile.clone(), tile, liquid, amount, context)
    }

    pub fn deposit(
        &mut self,
        tile: Option<PuddleTileView>,
        source: Option<PuddleTileView>,
        liquid: PuddleLiquidInfo,
        amount: f32,
        context: PuddleDepositContext,
    ) -> PuddleDepositResult {
        let Some(tile) = tile else {
            return PuddleDepositResult::ignored(None);
        };
        let key = (tile.x, tile.y);

        if !self.in_bounds(tile.x, tile.y) {
            return PuddleDepositResult::ignored(Some(key));
        }

        let source = source.unwrap_or_else(|| tile.clone());
        let ax = (tile.world_x() + source.world_x()) / 2.0;
        let ay = (tile.world_y() + source.world_y()) / 2.0;

        if liquid.will_boil(context.heat_env) {
            let mut result = PuddleDepositResult::with_outcome(PuddleDepositOutcome::Boiled, key);
            result.vapor_effect = context.vapor_chance_passed;
            return result;
        }

        if context.space {
            let mut result = PuddleDepositResult::with_outcome(PuddleDepositOutcome::Space, key);
            result.space_liquid = context.space_liquid_chance_passed && !tile.same_tile(&source);
            return result;
        }

        if tile.floor_is_liquid {
            if let Some(floor_liquid) = &tile.floor_liquid {
                if !liquid.can_stay_on(floor_liquid) {
                    let reaction = react_puddle(floor_liquid, &liquid, amount, context);
                    let mut result = PuddleDepositResult::with_outcome(
                        PuddleDepositOutcome::ReactedWithFloor,
                        key,
                    )
                    .apply_reaction(reaction);

                    if context.initial {
                        result.ripple = self.ripple_existing(key, context.time);
                    }

                    return result;
                }
            }
        }

        if tile.floor_solid {
            return PuddleDepositResult::ignored(Some(key));
        }

        let existing_missing_liquid = self
            .puddles
            .get(&key)
            .is_some_and(|entry| entry.puddle.liquid.is_none());
        if !self.puddles.contains_key(&key) || existing_missing_liquid {
            if context.net_client {
                return PuddleDepositResult::ignored(Some(key));
            }

            let mut puddle = PuddleComp::new(self.alloc_id(), ax, ay);
            puddle.tile = Some(tile.to_component_tile());
            puddle.liquid = Some(liquid.to_component_liquid());
            puddle.amount = amount.min(MAX_LIQUID);
            puddle.added = true;
            puddle.registered = true;

            let mut result = PuddleDepositResult::with_outcome(PuddleDepositOutcome::Created, key);
            result.created = true;
            result.amount = puddle.amount;
            self.puddles.insert(key, PuddleEntry { puddle, liquid });
            return result;
        }

        let entry = self.puddles.get_mut(&key).expect("puddle key checked");
        if entry.liquid.same_liquid(&liquid) {
            entry.puddle.accepting = entry.puddle.accepting.max(amount);
            let mut ripple = false;
            if context.initial
                && entry.puddle.last_ripple <= context.time - 40.0
                && entry.puddle.amount >= MAX_LIQUID / 2.0
            {
                entry.puddle.last_ripple = context.time;
                ripple = true;
            }

            let mut result = PuddleDepositResult::with_outcome(PuddleDepositOutcome::Accepted, key);
            result.ripple = ripple;
            result.amount = entry.puddle.amount;
            result.accepting = entry.puddle.accepting;
            return result;
        }

        let reaction = react_puddle(&entry.liquid, &liquid, amount, context);
        let mut added = reaction.added;
        if context.cap {
            added = cap_reaction_amount(MAX_LIQUID - entry.puddle.amount, added);
        }
        entry.puddle.amount += added;

        let mut result = PuddleDepositResult::with_outcome(PuddleDepositOutcome::Mixed, key)
            .apply_reaction(reaction);
        result.added = added;
        result.amount = entry.puddle.amount;
        result
    }

    fn ripple_existing(&mut self, key: (i32, i32), time: f32) -> bool {
        if let Some(entry) = self.puddles.get_mut(&key) {
            if entry.puddle.last_ripple <= time - 40.0 {
                entry.puddle.last_ripple = time;
                return true;
            }
        }
        false
    }

    fn alloc_id(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }
}

impl Default for Puddles {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

pub fn react_puddle(
    dest: &PuddleLiquidInfo,
    liquid: &PuddleLiquidInfo,
    amount: f32,
    context: PuddleDepositContext,
) -> PuddleReactionResult {
    if (dest.flammability > 0.3 && liquid.temperature > 0.7)
        || (liquid.flammability > 0.3 && dest.temperature > 0.7)
    {
        PuddleReactionResult {
            added: dest.react_added(liquid, amount),
            create_fire: true,
            fireball: context.fireball_chance_passed,
            steam: false,
        }
    } else if dest.temperature > 0.7 && liquid.temperature < 0.55 {
        PuddleReactionResult {
            added: -0.1 * amount,
            create_fire: false,
            fireball: false,
            steam: context.steam_chance_passed,
        }
    } else if liquid.temperature > 0.7 && dest.temperature < 0.55 {
        PuddleReactionResult {
            added: -0.7 * amount,
            create_fire: false,
            fireball: false,
            steam: context.steam_chance_passed,
        }
    } else {
        PuddleReactionResult {
            added: dest.react_added(liquid, amount),
            create_fire: false,
            fireball: false,
            steam: false,
        }
    }
}

fn cap_reaction_amount(remaining: f32, added: f32) -> f32 {
    if added > 0.0 {
        added.min(remaining.max(0.0))
    } else {
        added
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water() -> PuddleLiquidInfo {
        let mut liquid = PuddleLiquidInfo::new("water");
        liquid.boil_point = 0.5;
        liquid
    }

    fn oil() -> PuddleLiquidInfo {
        let mut liquid = PuddleLiquidInfo::new("oil");
        liquid.flammability = 1.2;
        liquid.viscosity = 0.75;
        liquid.boil_point = 0.65;
        liquid.can_stay_on.push("water".to_string());
        liquid
    }

    fn slag() -> PuddleLiquidInfo {
        let mut liquid = PuddleLiquidInfo::new("slag");
        liquid.temperature = 1.0;
        liquid.viscosity = 0.7;
        liquid
    }

    fn cryofluid() -> PuddleLiquidInfo {
        let mut liquid = PuddleLiquidInfo::new("cryofluid");
        liquid.temperature = 0.25;
        liquid.boil_point = 0.55;
        liquid
    }

    #[test]
    fn deposit_creates_puddle_on_server_and_ignores_clients() {
        let tile = PuddleTileView::new(2, 3);
        let mut puddles = Puddles::new(10, 10);

        let result = puddles.deposit_at(
            Some(tile.clone()),
            water(),
            90.0,
            PuddleDepositContext::default(),
        );

        assert_eq!(result.outcome, PuddleDepositOutcome::Created);
        assert!(result.created);
        assert_eq!(result.amount, MAX_LIQUID);
        assert_eq!(puddles.len(), 1);
        let puddle = puddles.get(2, 3).unwrap();
        assert_eq!(puddle.tile, Some(tile.to_component_tile()));
        assert_eq!((puddle.x, puddle.y), (16.0, 24.0));

        let mut client_puddles = Puddles::new(10, 10);
        let ignored = client_puddles.deposit_at(
            Some(tile),
            water(),
            10.0,
            PuddleDepositContext {
                net_client: true,
                ..PuddleDepositContext::default()
            },
        );
        assert_eq!(ignored.outcome, PuddleDepositOutcome::Ignored);
        assert!(client_puddles.is_empty());
    }

    #[test]
    fn deposit_same_liquid_updates_accepting_and_ripple_cooldown() {
        let tile = PuddleTileView::new(1, 1);
        let mut puddles = Puddles::new(5, 5);
        puddles.deposit_at(
            Some(tile.clone()),
            water(),
            40.0,
            PuddleDepositContext::default(),
        );
        puddles.get_mut(1, 1).unwrap().last_ripple = 10.0;

        let result = puddles.deposit_at(
            Some(tile),
            water(),
            12.0,
            PuddleDepositContext {
                time: 55.0,
                ..PuddleDepositContext::default()
            },
        );

        assert_eq!(result.outcome, PuddleDepositOutcome::Accepted);
        assert_eq!(result.accepting, 12.0);
        assert!(result.ripple);
        assert_eq!(puddles.get(1, 1).unwrap().last_ripple, 55.0);
    }

    #[test]
    fn slurp_matching_liquid_decrements_only_matching_puddle() {
        let tile = PuddleTileView::new(1, 1);
        let mut puddles = Puddles::new(5, 5);
        puddles.deposit_at(Some(tile), water(), 30.0, PuddleDepositContext::default());

        assert_eq!(puddles.slurp_matching_liquid(1, 1, "oil", 5.0), 0.0);
        assert_eq!(puddles.get(1, 1).unwrap().amount, 30.0);
        assert_eq!(puddles.slurp_matching_liquid(1, 1, "water", 12.0), 12.0);
        assert_eq!(puddles.get(1, 1).unwrap().amount, 18.0);
    }

    #[test]
    fn boil_and_space_branches_short_circuit_without_creating_puddles() {
        let tile = PuddleTileView::new(1, 1);
        let source = PuddleTileView::new(0, 1);
        let mut puddles = Puddles::new(5, 5);

        let boiled = puddles.deposit_at(
            Some(tile.clone()),
            water(),
            10.0,
            PuddleDepositContext {
                heat_env: 0.5,
                vapor_chance_passed: true,
                ..PuddleDepositContext::default()
            },
        );
        assert_eq!(boiled.outcome, PuddleDepositOutcome::Boiled);
        assert!(boiled.vapor_effect);

        let space = puddles.deposit(
            Some(tile),
            Some(source),
            oil(),
            10.0,
            PuddleDepositContext {
                space: true,
                space_liquid_chance_passed: true,
                ..PuddleDepositContext::default()
            },
        );
        assert_eq!(space.outcome, PuddleDepositOutcome::Space);
        assert!(space.space_liquid);
        assert!(puddles.is_empty());
    }

    #[test]
    fn deposit_on_incompatible_liquid_floor_reacts_instead_of_creating_puddle() {
        let tile = PuddleTileView::new(2, 2).with_liquid_floor(water());
        let mut puddles = Puddles::new(5, 5);

        let result = puddles.deposit_at(
            Some(tile),
            slag(),
            10.0,
            PuddleDepositContext {
                steam_chance_passed: true,
                ..PuddleDepositContext::default()
            },
        );

        assert_eq!(result.outcome, PuddleDepositOutcome::ReactedWithFloor);
        assert_eq!(result.added, -7.0);
        assert!(result.steam);
        assert!(puddles.is_empty());
    }

    #[test]
    fn liquid_floor_allows_can_stay_on_liquid_to_create_normal_puddle() {
        let tile = PuddleTileView::new(2, 2).with_liquid_floor(water());
        let mut puddles = Puddles::new(5, 5);

        let result = puddles.deposit_at(Some(tile), oil(), 10.0, PuddleDepositContext::default());

        assert_eq!(result.outcome, PuddleDepositOutcome::Created);
        assert_eq!(puddles.len(), 1);
        assert!(puddles.has_liquid(Some(&PuddleTileView::new(2, 2)), &oil()));
    }

    #[test]
    fn mixed_liquids_apply_java_reaction_rules_and_cap_positive_additions() {
        let tile = PuddleTileView::new(1, 1);
        let mut neoplasm = PuddleLiquidInfo::new("neoplasm");
        neoplasm.reaction_target = Some("water".to_string());
        let mut puddles = Puddles::new(5, 5);
        puddles.deposit_at(
            Some(tile.clone()),
            neoplasm.clone(),
            68.0,
            PuddleDepositContext::default(),
        );

        let capped = puddles.deposit_at(
            Some(tile.clone()),
            water(),
            10.0,
            PuddleDepositContext {
                cap: true,
                ..PuddleDepositContext::default()
            },
        );
        assert_eq!(capped.outcome, PuddleDepositOutcome::Mixed);
        assert_eq!(capped.added, 2.0);
        assert_eq!(capped.amount, MAX_LIQUID);

        let hot_on_cold = react_puddle(
            &cryofluid(),
            &slag(),
            10.0,
            PuddleDepositContext {
                steam_chance_passed: true,
                ..PuddleDepositContext::default()
            },
        );
        assert_eq!(hot_on_cold.added, -7.0);
        assert!(hot_on_cold.steam);

        let fire = react_puddle(
            &oil(),
            &slag(),
            10.0,
            PuddleDepositContext {
                fireball_chance_passed: true,
                ..PuddleDepositContext::default()
            },
        );
        assert_eq!(fire.added, 0.0);
        assert!(fire.create_fire);
        assert!(fire.fireball);
    }

    #[test]
    fn remove_and_register_roundtrip_tile_entries() {
        let tile = PuddleTileView::new(3, 4);
        let mut puddles = Puddles::new(8, 8);
        puddles.deposit_at(
            Some(tile.clone()),
            water(),
            12.0,
            PuddleDepositContext::default(),
        );

        let removed = puddles.remove(Some(&tile)).unwrap();
        assert!(puddles.get(3, 4).is_none());
        assert!(puddles.register(removed.puddle, removed.liquid));
        assert!(puddles.has_liquid(Some(&tile), &water()));
    }
}
