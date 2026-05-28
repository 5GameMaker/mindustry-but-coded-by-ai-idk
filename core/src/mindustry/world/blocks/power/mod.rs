use std::{
    collections::{BTreeSet, VecDeque},
    io::{self, Read, Write},
};

pub mod power_block;
pub mod power_distributor;

pub use power_block::PowerBlock;
pub use power_distributor::PowerDistributor;

pub fn battery_status(power_status: f32) -> PowerBlockStatus {
    if nearly(power_status, 0.0, 0.001) {
        PowerBlockStatus::NoInput
    } else if nearly(power_status, 1.0, 0.001) {
        PowerBlockStatus::Active
    } else {
        PowerBlockStatus::NoOutput
    }
}

pub fn battery_overwrite_status(
    current_status: f32,
    incoming_capacity: f32,
    incoming_status: f32,
    battery_capacity: f32,
) -> f32 {
    (current_status + incoming_capacity * incoming_status / battery_capacity).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerBlockStatus {
    NoInput,
    Active,
    NoOutput,
    LogicDisable,
}

pub fn power_node_link_valid(
    same_building: bool,
    link_exists: bool,
    link_has_power: bool,
    link_connected_power: bool,
    same_team: bool,
    same_block_connection: bool,
    same_block: bool,
    overlaps_either_range: bool,
    check_max_nodes: bool,
    link_is_power_node: bool,
    link_count: i32,
    link_max_nodes: i32,
    already_linked: bool,
) -> bool {
    if same_building
        || !link_exists
        || !link_has_power
        || !link_connected_power
        || !same_team
        || (same_block_connection && !same_block)
        || !overlaps_either_range
    {
        return false;
    }
    if check_max_nodes && link_is_power_node {
        link_count < link_max_nodes || already_linked
    } else {
        true
    }
}

pub fn power_node_should_draw_link(
    link_valid: bool,
    link_is_power_node: bool,
    link_id: i32,
    self_id: i32,
) -> bool {
    link_valid && !(link_is_power_node && link_id >= self_id)
}

pub fn power_diode_bar(stored: f32, capacity: f32) -> f32 {
    if capacity == 0.0 {
        0.0
    } else {
        stored / capacity
    }
}

pub fn power_diode_transfer_amount(
    back_stored: f32,
    back_capacity: f32,
    front_stored: f32,
    front_capacity: f32,
) -> f32 {
    if back_capacity <= 0.0 || front_capacity <= 0.0 {
        return 0.0;
    }
    if back_stored / back_capacity <= front_stored / front_capacity {
        return 0.0;
    }
    let target_percentage = (front_stored + back_stored) / (front_capacity + back_capacity);
    ((target_percentage * front_capacity - front_stored) / 2.0)
        .clamp(0.0, front_capacity - front_stored)
}

pub fn solar_generator_efficiency(
    enabled: bool,
    solar_multiplier: f32,
    light_env: f32,
    lighting: bool,
    ambient_alpha: f32,
) -> f32 {
    if enabled {
        solar_multiplier * (light_env + if lighting { 1.0 - ambient_alpha } else { 1.0 }).max(0.0)
    } else {
        0.0
    }
}

pub fn thermal_generator_can_place(attribute_sum: f32, min_efficiency: f32) -> bool {
    attribute_sum > min_efficiency
}

pub fn thermal_generator_efficiency(attribute_sum: f32, env: f32) -> f32 {
    attribute_sum + env
}

pub fn thermal_generator_liquid_added(
    production_efficiency: f32,
    delta: f32,
    output_amount: f32,
    liquid_capacity: f32,
    stored: f32,
) -> f32 {
    (production_efficiency * delta * output_amount)
        .min(liquid_capacity - stored)
        .max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerGeneratorState {
    pub production_efficiency: f32,
    pub generate_time: f32,
}

impl Default for PowerGeneratorState {
    fn default() -> Self {
        Self {
            production_efficiency: 0.0,
            generate_time: 0.0,
        }
    }
}

pub fn power_generator_production(
    enabled: bool,
    power_production: f32,
    production_efficiency: f32,
) -> f32 {
    if enabled {
        power_production * production_efficiency
    } else {
        0.0
    }
}

pub fn power_generator_warmup(enabled: bool, production_efficiency: f32) -> f32 {
    if enabled {
        production_efficiency
    } else {
        0.0
    }
}

pub fn power_generator_ambient_volume(production_efficiency: f32) -> f32 {
    production_efficiency.clamp(0.0, 1.0)
}

pub fn power_generator_should_explode(warmup: f32, explosion_min_warmup: f32) -> bool {
    warmup >= explosion_min_warmup
}

pub fn write_power_generator_state<W: Write>(
    write: &mut W,
    state: &PowerGeneratorState,
) -> io::Result<()> {
    write_f32(write, state.production_efficiency)?;
    write_f32(write, state.generate_time)
}

pub fn read_power_generator_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<PowerGeneratorState> {
    Ok(PowerGeneratorState {
        production_efficiency: read_f32(read)?,
        generate_time: if revision >= 1 { read_f32(read)? } else { 0.0 },
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeGeneratorState {
    pub warmup: f32,
    pub total_time: f32,
    pub efficiency_multiplier: f32,
    pub item_duration_multiplier: f32,
    pub generate_time: f32,
}

impl Default for ConsumeGeneratorState {
    fn default() -> Self {
        Self {
            warmup: 0.0,
            total_time: 0.0,
            efficiency_multiplier: 1.0,
            item_duration_multiplier: 1.0,
            generate_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeGeneratorUpdate {
    pub should_consume: bool,
    pub liquid_added: f32,
    pub exploded: bool,
    pub production_efficiency: f32,
    pub generate_time: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn consume_generator_update(
    state: &mut ConsumeGeneratorState,
    efficiency: f32,
    warmup_speed: f32,
    delta: f32,
    has_items: bool,
    item_duration: f32,
    output_liquid_amount: Option<f32>,
    liquid_capacity: f32,
    stored_liquid: f32,
    explode_on_full: bool,
) -> ConsumeGeneratorUpdate {
    let valid = efficiency > 0.0;
    state.warmup = lerp_delta(state.warmup, if valid { 1.0 } else { 0.0 }, warmup_speed);
    let production_efficiency = efficiency * state.efficiency_multiplier;
    state.total_time += state.warmup * delta;

    let should_consume = has_items && valid && state.generate_time <= 0.0;
    if should_consume {
        state.generate_time = 1.0;
    }

    let liquid_added = output_liquid_amount
        .map(|amount| {
            thermal_generator_liquid_added(
                production_efficiency,
                delta,
                amount,
                liquid_capacity,
                stored_liquid,
            )
        })
        .unwrap_or(0.0);
    let after_liquid = stored_liquid + liquid_added;
    let exploded =
        explode_on_full && output_liquid_amount.is_some() && after_liquid >= liquid_capacity - 0.01;

    state.generate_time -= delta / (item_duration * state.item_duration_multiplier);

    ConsumeGeneratorUpdate {
        should_consume,
        liquid_added,
        exploded,
        production_efficiency,
        generate_time: state.generate_time,
    }
}

pub fn consume_generator_trigger_valid(generate_time: f32) -> bool {
    generate_time > 0.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NuclearReactorState {
    pub generator: PowerGeneratorState,
    pub heat: f32,
}

pub fn nuclear_fuel_fullness(fuel: f32, item_capacity: f32) -> f32 {
    if item_capacity <= 0.0 {
        0.0
    } else {
        fuel / item_capacity
    }
}

pub fn nuclear_heat_gain(fullness: f32, heating: f32, delta: f32) -> f32 {
    fullness * heating * delta.min(4.0)
}

pub fn nuclear_cooldown_heat(heat: f32, ambient_cooldown_time: f32, delta: f32) -> f32 {
    (heat - delta / ambient_cooldown_time).max(0.0)
}

pub fn nuclear_coolant_absorb(heat: f32, coolant_power: f32, liquid_amount: f32) -> (f32, f32) {
    if heat <= 0.0 || coolant_power <= 0.0 || liquid_amount <= 0.0 {
        return (heat, 0.0);
    }
    let used = liquid_amount.min(heat / coolant_power);
    (heat - used * coolant_power, used)
}

pub fn nuclear_smoke_chance(heat: f32, smoke_threshold: f32, delta: f32) -> f32 {
    if heat > smoke_threshold {
        let smoke = 1.0 + (heat - smoke_threshold) / (1.0 - smoke_threshold);
        smoke / 20.0 * delta
    } else {
        0.0
    }
}

pub fn nuclear_heat_progress(
    current: f32,
    heat: f32,
    heat_output: f32,
    enabled: bool,
    heat_warmup_rate: f32,
    delta: f32,
) -> f32 {
    if heat_output > 0.0 {
        approach_delta(
            current,
            heat.clamp(0.0, 1.0) * heat_output * if enabled { 1.0 } else { 0.0 },
            heat_warmup_rate * delta,
        )
    } else {
        0.0
    }
}

pub fn nuclear_should_overheat(heat: f32) -> bool {
    heat >= 0.999
}

pub fn nuclear_should_explode(base_should_explode: bool, fuel_count: i32, heat: f32) -> bool {
    base_should_explode && (fuel_count >= 5 || heat >= 0.5)
}

pub fn write_nuclear_reactor_state<W: Write>(
    write: &mut W,
    state: &NuclearReactorState,
) -> io::Result<()> {
    write_power_generator_state(write, &state.generator)?;
    write_f32(write, state.heat)
}

pub fn read_nuclear_reactor_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<NuclearReactorState> {
    Ok(NuclearReactorState {
        generator: read_power_generator_state(read, revision)?,
        heat: read_f32(read)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImpactReactorState {
    pub generator: PowerGeneratorState,
    pub warmup: f32,
}

pub fn impact_reactor_triggered(efficiency: f32, power_status: f32) -> bool {
    efficiency >= 0.9999 && power_status >= 0.99
}

pub fn impact_reactor_warmup(current: f32, warmup_speed: f32, time_scale: f32) -> f32 {
    let next = lerp_delta(current, 1.0, warmup_speed * time_scale);
    if nearly(next, 1.0, 0.001) {
        1.0
    } else {
        next
    }
}

pub fn impact_reactor_cooldown(current: f32) -> f32 {
    lerp_delta(current, 0.0, 0.01)
}

pub fn impact_reactor_efficiency(warmup: f32) -> f32 {
    warmup.powf(5.0)
}

pub fn write_impact_reactor_state<W: Write>(
    write: &mut W,
    state: &ImpactReactorState,
) -> io::Result<()> {
    write_power_generator_state(write, &state.generator)?;
    write_f32(write, state.warmup)
}

pub fn read_impact_reactor_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<ImpactReactorState> {
    Ok(ImpactReactorState {
        generator: read_power_generator_state(read, revision)?,
        warmup: read_f32(read)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VariableReactorState {
    pub generator: PowerGeneratorState,
    pub heat: f32,
    pub instability: f32,
    pub warmup: f32,
}

pub fn variable_reactor_target(heat: f32, max_heat: f32) -> f32 {
    if max_heat <= 0.0 {
        0.0
    } else {
        (heat / max_heat).clamp(0.0, 1.0)
    }
}

pub fn variable_reactor_efficiency_met(efficiency: f32, target: f32) -> f32 {
    if nearly(target, 0.0, 0.000001) {
        1.0
    } else {
        (efficiency / target).clamp(0.0, 1.0)
    }
}

pub fn variable_reactor_instability(
    instability: f32,
    efficiency_met: f32,
    unstable_speed: f32,
) -> f32 {
    let met = efficiency_met >= 0.99999;
    approach_delta(
        instability,
        if met { 0.0 } else { 1.0 },
        if met {
            0.5
        } else {
            unstable_speed * (1.0 - efficiency_met)
        },
    )
}

pub fn variable_reactor_production_efficiency(efficiency: f32, target: f32) -> f32 {
    efficiency * target
}

pub fn variable_reactor_warmup(warmup: f32, production_efficiency: f32, warmup_speed: f32) -> f32 {
    lerp_delta(
        warmup,
        if production_efficiency > 0.0 {
            1.0
        } else {
            0.0
        },
        warmup_speed,
    )
}

pub fn variable_reactor_should_explode(heat: f32) -> bool {
    heat > 0.0
}

pub fn write_variable_reactor_state<W: Write>(
    write: &mut W,
    state: &VariableReactorState,
) -> io::Result<()> {
    write_power_generator_state(write, &state.generator)?;
    write_f32(write, state.heat)?;
    write_f32(write, state.instability)?;
    write_f32(write, state.warmup)
}

pub fn read_variable_reactor_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<VariableReactorState> {
    Ok(VariableReactorState {
        generator: read_power_generator_state(read, revision)?,
        heat: read_f32(read)?,
        instability: read_f32(read)?,
        warmup: read_f32(read)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeaterGeneratorState {
    pub generator: PowerGeneratorState,
    pub heat: f32,
}

pub fn heater_generator_heat(
    current_heat: f32,
    heat_output: f32,
    efficiency: f32,
    warmup_rate: f32,
    delta: f32,
) -> f32 {
    approach_delta(current_heat, heat_output * efficiency, warmup_rate * delta)
}

pub fn heater_generator_heat_frac(heat: f32, heat_output: f32) -> f32 {
    if heat_output == 0.0 {
        0.0
    } else {
        heat / heat_output
    }
}

pub fn write_heater_generator_state<W: Write>(
    write: &mut W,
    state: &HeaterGeneratorState,
) -> io::Result<()> {
    write_power_generator_state(write, &state.generator)?;
    write_f32(write, state.heat)
}

pub fn read_heater_generator_state<R: Read>(
    read: &mut R,
    revision: i32,
) -> io::Result<HeaterGeneratorState> {
    Ok(HeaterGeneratorState {
        generator: read_power_generator_state(read, revision)?,
        heat: read_f32(read)?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightBlockState {
    pub color: i32,
}

pub fn light_block_place_radius2(radius: f32, tilesize: f32) -> f32 {
    (radius * 0.7 / tilesize).powf(2.0) * 3.0
}

pub fn light_block_minimap_color(color: i32) -> i32 {
    color | 0xff
}

pub fn light_block_minimap_color_rgba(color: u32) -> u32 {
    color | 0xff
}

pub fn light_block_smooth_time(smooth_time: f32, time_scale: f32) -> f32 {
    lerp_delta(smooth_time, time_scale, 0.1)
}

pub fn light_block_intensity(brightness: f32, efficiency: f32) -> f32 {
    brightness * efficiency
}

pub fn write_light_block_state<W: Write>(write: &mut W, state: &LightBlockState) -> io::Result<()> {
    write_i32(write, state.color)
}

pub fn read_light_block_state<R: Read>(read: &mut R) -> io::Result<LightBlockState> {
    Ok(LightBlockState {
        color: read_i32(read)?,
    })
}

pub fn write_consume_generator_state<W: Write>(
    write: &mut W,
    state: &ConsumeGeneratorState,
) -> io::Result<()> {
    write_f32(write, state.warmup)?;
    write_f32(write, state.total_time)?;
    write_f32(write, state.efficiency_multiplier)?;
    write_f32(write, state.item_duration_multiplier)?;
    write_f32(write, state.generate_time)
}

pub fn read_consume_generator_state<R: Read>(read: &mut R) -> io::Result<ConsumeGeneratorState> {
    Ok(ConsumeGeneratorState {
        warmup: read_f32(read)?,
        total_time: read_f32(read)?,
        efficiency_multiplier: read_f32(read)?,
        item_duration_multiplier: read_f32(read)?,
        generate_time: read_f32(read)?,
    })
}

fn nearly(value: f32, target: f32, epsilon: f32) -> bool {
    (value - target).abs() <= epsilon
}

fn lerp_delta(from: f32, to: f32, alpha: f32) -> f32 {
    from + (to - from) * alpha
}

fn approach_delta(from: f32, to: f32, amount: f32) -> f32 {
    if from < to {
        (from + amount).min(to)
    } else {
        (from - amount).max(to)
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerBattery {
    pub status: f32,
    pub capacity: f32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerProducer {
    pub production: f32,
    pub delta: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerConsumer {
    pub should_consume_power: bool,
    pub requested_power: f32,
    pub usage: f32,
    pub delta: f32,
    pub buffered: bool,
    pub capacity: f32,
    pub status: f32,
    pub cheating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerGraphNode {
    pub id: i32,
    pub outputs_power: bool,
    pub consumes_power: bool,
    pub buffered: bool,
    pub should_consume_power: bool,
    pub production: f32,
    pub requested_power: f32,
    pub usage: f32,
    pub delta: f32,
    pub battery_status: f32,
    pub battery_capacity: f32,
    pub enabled: bool,
    pub cheating: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PowerGraphRuntime {
    pub all: Vec<i32>,
    pub producers: Vec<PowerProducer>,
    pub producer_nodes: Vec<i32>,
    pub consumers: Vec<PowerConsumer>,
    pub consumer_nodes: Vec<i32>,
    pub batteries: Vec<PowerBattery>,
    pub battery_nodes: Vec<i32>,
    pub last_power_produced: f32,
    pub last_power_needed: f32,
    pub last_power_stored: f32,
    pub last_scaled_power_in: f32,
    pub last_scaled_power_out: f32,
    pub last_capacity: f32,
    pub energy_delta: f32,
    power_balance_samples: Vec<f32>,
    power_balance_window: usize,
}

impl Default for PowerGraphRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerGraphRuntime {
    pub fn new() -> Self {
        Self {
            all: Vec::new(),
            producers: Vec::new(),
            producer_nodes: Vec::new(),
            consumers: Vec::new(),
            consumer_nodes: Vec::new(),
            batteries: Vec::new(),
            battery_nodes: Vec::new(),
            last_power_produced: 0.0,
            last_power_needed: 0.0,
            last_power_stored: 0.0,
            last_scaled_power_in: 0.0,
            last_scaled_power_out: 0.0,
            last_capacity: 0.0,
            energy_delta: 0.0,
            power_balance_samples: Vec::new(),
            power_balance_window: 60,
        }
    }

    pub fn add_node(&mut self, node: PowerGraphNode) -> bool {
        if self.all.contains(&node.id) {
            return false;
        }
        self.all.push(node.id);
        if node.outputs_power && node.consumes_power && !node.buffered {
            self.producer_nodes.push(node.id);
            self.producers.push(PowerProducer {
                production: node.production,
                delta: node.delta,
            });
            self.consumer_nodes.push(node.id);
            self.consumers.push(PowerConsumer {
                should_consume_power: node.should_consume_power,
                requested_power: node.requested_power,
                usage: node.usage,
                delta: node.delta,
                buffered: node.buffered,
                capacity: node.battery_capacity,
                status: node.battery_status,
                cheating: node.cheating,
            });
        } else if node.outputs_power && node.consumes_power {
            self.battery_nodes.push(node.id);
            self.batteries.push(PowerBattery {
                status: node.battery_status,
                capacity: node.battery_capacity,
                enabled: node.enabled,
            });
        } else if node.outputs_power {
            self.producer_nodes.push(node.id);
            self.producers.push(PowerProducer {
                production: node.production,
                delta: node.delta,
            });
        } else if node.consumes_power {
            self.consumer_nodes.push(node.id);
            self.consumers.push(PowerConsumer {
                should_consume_power: node.should_consume_power,
                requested_power: node.requested_power,
                usage: node.usage,
                delta: node.delta,
                buffered: node.buffered,
                capacity: node.battery_capacity,
                status: node.battery_status,
                cheating: node.cheating,
            });
        }
        true
    }

    pub fn remove_list(&mut self, node_id: i32) -> bool {
        let mut removed = remove_first_value(&mut self.all, node_id);
        removed |= remove_indexed(&mut self.producer_nodes, &mut self.producers, node_id);
        removed |= remove_indexed(&mut self.consumer_nodes, &mut self.consumers, node_id);
        removed |= remove_indexed(&mut self.battery_nodes, &mut self.batteries, node_id);
        removed
    }

    pub fn clear(&mut self) {
        self.all.clear();
        self.producers.clear();
        self.producer_nodes.clear();
        self.consumers.clear();
        self.consumer_nodes.clear();
        self.batteries.clear();
        self.battery_nodes.clear();
    }

    pub fn add_graph(&mut self, other: &mut Self) {
        let other_ids = other.all.clone();
        for id in other_ids {
            if let Some(index) = other.producer_nodes.iter().position(|node| *node == id) {
                if !self.all.contains(&id) {
                    self.all.push(id);
                    self.producer_nodes.push(id);
                    self.producers.push(other.producers[index]);
                }
            }
            if let Some(index) = other.consumer_nodes.iter().position(|node| *node == id) {
                if !self.consumer_nodes.contains(&id) {
                    if !self.all.contains(&id) {
                        self.all.push(id);
                    }
                    self.consumer_nodes.push(id);
                    self.consumers.push(other.consumers[index]);
                }
            }
            if let Some(index) = other.battery_nodes.iter().position(|node| *node == id) {
                if !self.battery_nodes.contains(&id) {
                    if !self.all.contains(&id) {
                        self.all.push(id);
                    }
                    self.battery_nodes.push(id);
                    self.batteries.push(other.batteries[index]);
                }
            }
        }
        other.clear();
    }

    pub fn reflow_from<F>(&mut self, start: PowerGraphNode, mut connections: F)
    where
        F: FnMut(i32) -> Vec<PowerGraphNode>,
    {
        let mut queue = VecDeque::new();
        let mut closed = BTreeSet::new();
        closed.insert(start.id);
        queue.push_back(start);
        while let Some(node) = queue.pop_front() {
            let id = node.id;
            self.add_node(node);
            for next in connections(id) {
                if closed.insert(next.id) {
                    queue.push_back(next);
                }
            }
        }
    }

    pub fn remove_with_connections<C, L>(
        &mut self,
        removed_id: i32,
        mut connections: C,
        mut lookup: L,
    ) -> Vec<PowerGraphRuntime>
    where
        C: FnMut(i32) -> Vec<i32>,
        L: FnMut(i32) -> Option<PowerGraphNode>,
    {
        let original_members = self.all.iter().copied().collect::<BTreeSet<_>>();
        let mut assigned = BTreeSet::new();
        let mut branches = Vec::new();

        for start_id in connections(removed_id) {
            if start_id == removed_id
                || !original_members.contains(&start_id)
                || !assigned.insert(start_id)
            {
                continue;
            }

            let Some(start_node) = lookup(start_id) else {
                continue;
            };
            let mut branch = PowerGraphRuntime::new();
            let mut queue = VecDeque::new();
            queue.push_back(start_node);

            while let Some(node) = queue.pop_front() {
                let id = node.id;
                branch.add_node(node);
                for next_id in connections(id) {
                    if next_id == removed_id
                        || !original_members.contains(&next_id)
                        || !assigned.insert(next_id)
                    {
                        continue;
                    }
                    if let Some(next_node) = lookup(next_id) {
                        queue.push_back(next_node);
                    }
                }
            }

            branch.update_with_delta(1.0);
            branches.push(branch);
        }

        self.clear();
        branches
    }

    pub fn transfer_power(&mut self, amount: f32) {
        if amount > 0.0 {
            power_graph_charge_batteries(&mut self.batteries, amount);
        } else {
            power_graph_use_batteries(&mut self.batteries, -amount);
        }
        self.energy_delta += amount;
    }

    pub fn power_balance(&self) -> f32 {
        if self.power_balance_samples.is_empty() {
            0.0
        } else {
            self.power_balance_samples.iter().sum::<f32>() / self.power_balance_samples.len() as f32
        }
    }

    pub fn has_power_balance_samples(&self) -> bool {
        self.power_balance_samples.len() >= self.power_balance_window
    }

    pub fn update_with_delta(&mut self, time_delta: f32) {
        if self
            .consumers
            .first()
            .is_some_and(|consumer| consumer.cheating)
        {
            for consumer in &mut self.consumers {
                consumer.status = 1.0;
            }
            self.last_power_needed = 1.0;
            self.last_power_produced = 1.0;
            return;
        }

        let safe_delta = if nearly(time_delta, 0.0, 0.0001) {
            1.0
        } else {
            time_delta
        };
        let power_needed = power_graph_power_needed(
            &self
                .consumers
                .iter()
                .map(|consumer| {
                    (
                        consumer.should_consume_power,
                        consumer.requested_power,
                        consumer.delta,
                    )
                })
                .collect::<Vec<_>>(),
        );
        let mut power_produced = power_graph_power_produced(
            &self
                .producers
                .iter()
                .map(|producer| (producer.production, producer.delta))
                .collect::<Vec<_>>(),
        );

        self.last_power_needed = power_needed;
        self.last_power_produced = power_produced;
        self.last_scaled_power_in =
            power_graph_scaled_power_in(power_produced, self.energy_delta, safe_delta);
        self.last_scaled_power_out = power_graph_scaled_power_out(power_needed, safe_delta);
        self.last_capacity = power_graph_total_battery_capacity(&self.batteries);
        self.last_power_stored = power_graph_battery_stored(&self.batteries);
        self.push_power_balance_sample(
            (self.last_power_produced - self.last_power_needed + self.energy_delta) / safe_delta,
        );
        self.energy_delta = 0.0;

        if self.consumers.is_empty() && self.producers.is_empty() && self.batteries.is_empty() {
            return;
        }

        let mut charged = false;
        if !nearly(power_needed, power_produced, 0.0001) {
            if power_needed > power_produced {
                let used =
                    power_graph_use_batteries(&mut self.batteries, power_needed - power_produced);
                power_produced += used;
                self.last_power_produced += used;
            } else {
                charged = true;
                power_produced -= power_graph_charge_batteries(
                    &mut self.batteries,
                    power_produced - power_needed,
                );
            }
        }

        let coverage = power_graph_coverage(
            power_needed,
            power_produced,
            charged,
            self.last_power_stored,
        );
        for consumer in &mut self.consumers {
            if consumer.buffered {
                consumer.status = power_graph_buffered_status(
                    consumer.status,
                    consumer.requested_power,
                    coverage,
                    consumer.delta,
                    consumer.capacity,
                );
            } else {
                consumer.status = power_graph_unbuffered_status(
                    consumer.should_consume_power,
                    coverage,
                    power_produced,
                    power_needed,
                    consumer.usage,
                    consumer.delta,
                );
            }
        }
    }

    fn push_power_balance_sample(&mut self, sample: f32) {
        self.power_balance_samples.push(sample);
        if self.power_balance_samples.len() > self.power_balance_window {
            self.power_balance_samples.remove(0);
        }
    }
}

fn remove_first_value(values: &mut Vec<i32>, target: i32) -> bool {
    if let Some(index) = values.iter().position(|value| *value == target) {
        values.remove(index);
        true
    } else {
        false
    }
}

fn remove_indexed<T>(ids: &mut Vec<i32>, values: &mut Vec<T>, target: i32) -> bool {
    if let Some(index) = ids.iter().position(|value| *value == target) {
        ids.remove(index);
        values.remove(index);
        true
    } else {
        false
    }
}

pub fn power_graph_satisfaction(last_power_produced: f32, last_power_needed: f32) -> f32 {
    if nearly(last_power_produced, 0.0, 0.0001) {
        0.0
    } else if nearly(last_power_needed, 0.0, 0.0001) {
        1.0
    } else {
        (last_power_produced / last_power_needed).clamp(0.0, 1.0)
    }
}

pub fn power_graph_power_produced(producers: &[(f32, f32)]) -> f32 {
    producers
        .iter()
        .map(|(production, delta)| production * delta)
        .sum()
}

pub fn power_graph_power_needed(consumers: &[(bool, f32, f32)]) -> f32 {
    consumers
        .iter()
        .filter(|(should_consume, _, _)| *should_consume)
        .map(|(_, requested_power, delta)| requested_power * delta)
        .sum()
}

pub fn power_graph_battery_stored(batteries: &[PowerBattery]) -> f32 {
    batteries
        .iter()
        .filter(|battery| battery.enabled)
        .map(|battery| battery.status * battery.capacity)
        .sum()
}

pub fn power_graph_battery_capacity(batteries: &[PowerBattery]) -> f32 {
    batteries
        .iter()
        .filter(|battery| battery.enabled)
        .map(|battery| (1.0 - battery.status) * battery.capacity)
        .sum()
}

pub fn power_graph_total_battery_capacity(batteries: &[PowerBattery]) -> f32 {
    batteries
        .iter()
        .filter(|battery| battery.enabled)
        .map(|battery| battery.capacity)
        .sum()
}

pub fn power_graph_use_batteries(batteries: &mut [PowerBattery], needed: f32) -> f32 {
    let stored = power_graph_battery_stored(batteries);
    if nearly(stored, 0.0, 0.0001) {
        return 0.0;
    }
    let used = stored.min(needed);
    let consumed_power_percentage = (needed / stored).min(1.0);
    for battery in batteries.iter_mut().filter(|battery| battery.enabled) {
        battery.status *= 1.0 - consumed_power_percentage;
    }
    used
}

pub fn power_graph_charge_batteries(batteries: &mut [PowerBattery], excess: f32) -> f32 {
    let capacity = power_graph_battery_capacity(batteries);
    let charged_percent = (excess / capacity).min(1.0);
    if nearly(capacity, 0.0, 0.0001) {
        return 0.0;
    }
    for battery in batteries
        .iter_mut()
        .filter(|battery| battery.enabled && battery.capacity > 0.0)
    {
        battery.status += (1.0 - battery.status) * charged_percent;
    }
    excess.min(capacity)
}

pub fn power_graph_coverage(
    needed: f32,
    produced: f32,
    charged: bool,
    last_power_stored: f32,
) -> f32 {
    if nearly(needed, 0.0, 0.0001)
        && nearly(produced, 0.0, 0.0001)
        && !charged
        && nearly(last_power_stored, 0.0, 0.0001)
    {
        0.0
    } else if nearly(needed, 0.0, 0.0001) {
        1.0
    } else {
        (produced / needed).min(1.0)
    }
}

pub fn power_graph_buffered_status(
    current_status: f32,
    requested_power: f32,
    coverage: f32,
    delta: f32,
    capacity: f32,
) -> f32 {
    if nearly(capacity, 0.0, 0.0001) {
        current_status
    } else {
        (current_status + requested_power * coverage * delta / capacity).clamp(0.0, 1.0)
    }
}

pub fn power_graph_unbuffered_status(
    should_consume_power: bool,
    coverage: f32,
    produced: f32,
    needed: f32,
    usage: f32,
    delta: f32,
) -> f32 {
    if should_consume_power {
        coverage
    } else {
        let status = (produced / (needed + usage * delta)).min(1.0);
        if status.is_nan() {
            0.0
        } else {
            status
        }
    }
}

pub fn power_graph_scaled_power_in(power_produced: f32, energy_delta: f32, delta: f32) -> f32 {
    (power_produced + energy_delta) / delta
}

pub fn power_graph_scaled_power_out(power_needed: f32, delta: f32) -> f32 {
    power_needed / delta
}

pub fn beam_node_update_clip_radius(range: i32, tilesize: f32) -> f32 {
    (range + 1) as f32 * tilesize
}

pub fn beam_node_could_connect_scan_range(range: i32, size: i32) -> std::ops::RangeInclusive<i32> {
    let range_offset = size / 2;
    (1 + range_offset)..=(range + range_offset)
}

pub fn beam_node_within_target_rect(
    other_x: i32,
    other_y: i32,
    target_x: i32,
    target_y: i32,
    target_size: i32,
) -> bool {
    let offset = -(target_size - 1) / 2;
    let min_x = target_x + offset;
    let min_y = target_y + offset;
    let max_x = target_x + offset + target_size - 1;
    let max_y = target_y + offset + target_size - 1;
    other_x >= min_x && other_y >= min_y && other_x <= max_x && other_y <= max_y
}

pub fn beam_node_draw_laser_size_offset(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    size1: i32,
    size2: i32,
    tilesize: f32,
) -> f32 {
    let dst = (x1 - x2).abs().max((y2 - y1).abs()) / tilesize;
    dst * tilesize - (size1 + size2) as f32 * tilesize / 2.0
}

pub fn beam_node_should_draw_laser(dst_tiles: i32, size: i32) -> bool {
    dst_tiles > 1 + size / 2
}

pub fn beam_node_status(power_balance: f32, last_power_stored: f32) -> PowerBlockStatus {
    if power_balance > 0.0 {
        PowerBlockStatus::Active
    } else if power_balance < 0.0 && last_power_stored > 0.0 {
        PowerBlockStatus::NoOutput
    } else {
        PowerBlockStatus::NoInput
    }
}

pub fn long_power_node_warmup(warmup: f32, link_count: usize) -> f32 {
    lerp_delta(warmup, if link_count > 0 { 1.0 } else { 0.0 }, 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_status_overwrite_and_node_links_follow_upstream_rules() {
        assert_eq!(battery_status(0.0), PowerBlockStatus::NoInput);
        assert_eq!(battery_status(1.0), PowerBlockStatus::Active);
        assert_eq!(battery_status(0.5), PowerBlockStatus::NoOutput);
        assert_eq!(battery_overwrite_status(0.2, 50.0, 0.5, 100.0), 0.45);

        assert!(power_node_link_valid(
            false, true, true, true, true, false, false, true, true, true, 2, 3, false
        ));
        assert!(!power_node_link_valid(
            true, true, true, true, true, false, false, true, true, true, 2, 3, false
        ));
        assert!(!power_node_link_valid(
            false, true, true, true, true, true, false, true, true, true, 2, 3, false
        ));
        assert!(power_node_link_valid(
            false, true, true, true, true, false, false, true, true, true, 3, 3, true
        ));
        assert!(!power_node_link_valid(
            false, true, true, true, true, false, false, true, true, true, 3, 3, false
        ));
        assert!(power_node_should_draw_link(true, true, 1, 2));
        assert!(!power_node_should_draw_link(true, true, 3, 2));
    }

    #[test]
    fn diode_solar_and_thermal_formulae_match_java_shells() {
        assert_eq!(power_diode_bar(5.0, 10.0), 0.5);
        assert_eq!(power_diode_bar(5.0, 0.0), 0.0);
        assert_eq!(power_diode_transfer_amount(80.0, 100.0, 20.0, 100.0), 15.0);
        assert_eq!(power_diode_transfer_amount(20.0, 100.0, 80.0, 100.0), 0.0);

        assert_eq!(solar_generator_efficiency(true, 2.0, 0.1, true, 0.25), 1.7);
        assert_eq!(solar_generator_efficiency(true, 2.0, 0.1, false, 0.25), 2.2);
        assert_eq!(
            solar_generator_efficiency(false, 2.0, 0.1, false, 0.25),
            0.0
        );

        assert!(thermal_generator_can_place(0.1, 0.0));
        assert!(!thermal_generator_can_place(0.0, 0.0));
        assert_eq!(thermal_generator_efficiency(0.5, 0.25), 0.75);
        assert_eq!(thermal_generator_liquid_added(2.0, 3.0, 0.1, 1.0, 0.5), 0.5);
    }

    #[test]
    fn consume_generator_update_and_state_roundtrip_follow_runtime_order() {
        let mut state = ConsumeGeneratorState::default();
        let update = consume_generator_update(
            &mut state,
            1.0,
            0.05,
            1.0,
            true,
            120.0,
            Some(0.2),
            10.0,
            9.9,
            true,
        );
        assert!(update.should_consume);
        assert_eq!(state.warmup, 0.05);
        assert!((update.liquid_added - 0.1).abs() < 0.00001);
        assert!(update.exploded);
        assert!(consume_generator_trigger_valid(update.generate_time));

        let mut bytes = Vec::new();
        write_consume_generator_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_consume_generator_state(&mut bytes.as_slice()).unwrap(),
            state
        );
    }

    #[test]
    fn power_generator_parent_state_matches_java_revision_gate() {
        let state = PowerGeneratorState {
            production_efficiency: 0.5,
            generate_time: 0.25,
        };
        assert_eq!(power_generator_production(true, 6.0, 0.5), 3.0);
        assert_eq!(power_generator_production(false, 6.0, 0.5), 0.0);
        assert_eq!(power_generator_warmup(true, 0.75), 0.75);
        assert_eq!(power_generator_ambient_volume(2.0), 1.0);
        assert!(power_generator_should_explode(0.4, 0.4));

        let mut bytes = Vec::new();
        write_power_generator_state(&mut bytes, &state).unwrap();
        assert_eq!(
            bytes,
            [0.5f32.to_be_bytes(), 0.25f32.to_be_bytes()].concat()
        );
        assert_eq!(
            read_power_generator_state(&mut bytes.as_slice(), 1).unwrap(),
            state
        );
        assert_eq!(
            read_power_generator_state(&mut 0.5f32.to_be_bytes().as_slice(), 0).unwrap(),
            PowerGeneratorState {
                production_efficiency: 0.5,
                generate_time: 0.0,
            }
        );
    }

    #[test]
    fn nuclear_reactor_formulae_and_state_order_follow_upstream() {
        assert_eq!(nuclear_fuel_fullness(20.0, 40.0), 0.5);
        assert_eq!(nuclear_heat_gain(0.5, 0.01, 6.0), 0.02);
        assert_eq!(nuclear_cooldown_heat(0.5, 10.0, 2.0), 0.3);
        assert_eq!(nuclear_coolant_absorb(0.8, 0.2, 10.0), (0.0, 4.0));
        assert!((nuclear_smoke_chance(0.9, 0.5, 1.0) - 0.09).abs() < 0.00001);
        assert_eq!(nuclear_heat_progress(0.0, 0.5, 4.0, true, 0.15, 2.0), 0.3);
        assert!(nuclear_should_overheat(0.999));
        assert!(nuclear_should_explode(true, 5, 0.0));
        assert!(nuclear_should_explode(true, 0, 0.5));
        assert!(!nuclear_should_explode(false, 5, 1.0));

        let state = NuclearReactorState {
            generator: PowerGeneratorState {
                production_efficiency: 0.6,
                generate_time: 0.2,
            },
            heat: 0.7,
        };
        let mut bytes = Vec::new();
        write_nuclear_reactor_state(&mut bytes, &state).unwrap();
        assert_eq!(
            read_nuclear_reactor_state(&mut bytes.as_slice(), 1).unwrap(),
            state
        );
    }

    #[test]
    fn impact_variable_heater_and_light_helpers_match_java_shells() {
        assert!(impact_reactor_triggered(0.9999, 0.99));
        assert!(!impact_reactor_triggered(0.99, 1.0));
        assert_eq!(impact_reactor_warmup(0.0, 0.2, 1.0), 0.2);
        assert_eq!(impact_reactor_cooldown(1.0), 0.99);
        assert!((impact_reactor_efficiency(0.5) - 0.03125).abs() < 0.000001);

        let target = variable_reactor_target(60.0, 120.0);
        assert_eq!(target, 0.5);
        assert_eq!(variable_reactor_efficiency_met(0.25, target), 0.5);
        assert_eq!(variable_reactor_instability(0.2, 1.0, 0.01), 0.0);
        assert_eq!(variable_reactor_production_efficiency(0.8, 0.5), 0.4);
        assert_eq!(variable_reactor_warmup(0.0, 0.4, 0.05), 0.05);
        assert!(variable_reactor_should_explode(0.01));

        assert_eq!(heater_generator_heat(0.0, 10.0, 0.5, 0.1, 2.0), 0.2);
        assert_eq!(heater_generator_heat_frac(2.0, 8.0), 0.25);

        assert!((light_block_place_radius2(40.0, 8.0) - 36.75).abs() < 0.00001);
        assert_eq!(light_block_minimap_color(0x11223300), 0x112233ff);
        assert_eq!(light_block_minimap_color_rgba(0x11223300), 0x112233ff);
        assert_eq!(light_block_smooth_time(0.0, 2.0), 0.2);
        assert_eq!(light_block_intensity(0.75, 0.5), 0.375);
    }

    #[test]
    fn reactor_and_light_state_serialization_keeps_parent_then_child_order() {
        let generator = PowerGeneratorState {
            production_efficiency: 0.25,
            generate_time: 0.75,
        };

        let impact = ImpactReactorState {
            generator,
            warmup: 0.5,
        };
        let mut bytes = Vec::new();
        write_impact_reactor_state(&mut bytes, &impact).unwrap();
        assert_eq!(
            read_impact_reactor_state(&mut bytes.as_slice(), 1).unwrap(),
            impact
        );

        let variable = VariableReactorState {
            generator,
            heat: 0.2,
            instability: 0.3,
            warmup: 0.4,
        };
        let mut bytes = Vec::new();
        write_variable_reactor_state(&mut bytes, &variable).unwrap();
        assert_eq!(
            read_variable_reactor_state(&mut bytes.as_slice(), 1).unwrap(),
            variable
        );

        let heater = HeaterGeneratorState {
            generator,
            heat: 3.5,
        };
        let mut bytes = Vec::new();
        write_heater_generator_state(&mut bytes, &heater).unwrap();
        assert_eq!(
            read_heater_generator_state(&mut bytes.as_slice(), 1).unwrap(),
            heater
        );

        let light = LightBlockState { color: 0x12345678 };
        let mut bytes = Vec::new();
        write_light_block_state(&mut bytes, &light).unwrap();
        assert_eq!(bytes, 0x12345678i32.to_be_bytes());
        assert_eq!(
            read_light_block_state(&mut bytes.as_slice()).unwrap(),
            light
        );
    }

    #[test]
    fn power_graph_beam_and_long_node_helpers_follow_upstream() {
        assert_eq!(power_graph_satisfaction(0.0, 10.0), 0.0);
        assert_eq!(power_graph_satisfaction(10.0, 0.0), 1.0);
        assert_eq!(power_graph_satisfaction(4.0, 8.0), 0.5);
        assert_eq!(power_graph_power_produced(&[(2.0, 3.0), (4.0, 0.5)]), 8.0);
        assert_eq!(
            power_graph_power_needed(&[(true, 2.0, 3.0), (false, 10.0, 10.0)]),
            6.0
        );

        let mut batteries = vec![
            PowerBattery {
                status: 0.5,
                capacity: 100.0,
                enabled: true,
            },
            PowerBattery {
                status: 0.25,
                capacity: 40.0,
                enabled: true,
            },
            PowerBattery {
                status: 1.0,
                capacity: 1000.0,
                enabled: false,
            },
        ];
        assert_eq!(power_graph_battery_stored(&batteries), 60.0);
        assert_eq!(power_graph_battery_capacity(&batteries), 80.0);
        assert_eq!(power_graph_total_battery_capacity(&batteries), 140.0);
        assert_eq!(power_graph_use_batteries(&mut batteries, 30.0), 30.0);
        assert_eq!(batteries[0].status, 0.25);
        assert_eq!(batteries[1].status, 0.125);
        assert_eq!(power_graph_charge_batteries(&mut batteries, 20.0), 20.0);
        assert!((batteries[0].status - 0.38636363).abs() < 0.00001);

        assert_eq!(power_graph_coverage(0.0, 0.0, false, 0.0), 0.0);
        assert_eq!(power_graph_coverage(0.0, 2.0, false, 0.0), 1.0);
        assert_eq!(power_graph_coverage(10.0, 4.0, false, 0.0), 0.4);
        assert_eq!(power_graph_buffered_status(0.2, 10.0, 0.5, 2.0, 100.0), 0.3);
        assert_eq!(
            power_graph_unbuffered_status(true, 0.4, 1.0, 2.0, 3.0, 4.0),
            0.4
        );
        assert_eq!(
            power_graph_unbuffered_status(false, 0.4, 6.0, 2.0, 1.0, 2.0),
            1.0
        );
        assert_eq!(power_graph_scaled_power_in(6.0, 2.0, 4.0), 2.0);
        assert_eq!(power_graph_scaled_power_out(6.0, 3.0), 2.0);

        assert_eq!(beam_node_update_clip_radius(5, 8.0), 48.0);
        assert_eq!(
            beam_node_could_connect_scan_range(5, 2).collect::<Vec<_>>(),
            vec![2, 3, 4, 5, 6]
        );
        assert!(beam_node_within_target_rect(10, 10, 10, 10, 2));
        assert!(!beam_node_within_target_rect(12, 10, 10, 10, 2));
        assert_eq!(
            beam_node_draw_laser_size_offset(0.0, 0.0, 32.0, 0.0, 1, 2, 8.0),
            20.0
        );
        assert!(beam_node_should_draw_laser(3, 2));
        assert_eq!(beam_node_status(1.0, 0.0), PowerBlockStatus::Active);
        assert_eq!(beam_node_status(-1.0, 1.0), PowerBlockStatus::NoOutput);
        assert_eq!(beam_node_status(0.0, 0.0), PowerBlockStatus::NoInput);
        assert_eq!(long_power_node_warmup(0.0, 1), 0.05);
    }

    #[test]
    fn power_graph_runtime_update_uses_batteries_and_updates_consumers() {
        let mut graph = PowerGraphRuntime::new();
        graph.producers.push(PowerProducer {
            production: 4.0,
            delta: 1.0,
        });
        graph.consumers.push(PowerConsumer {
            should_consume_power: true,
            requested_power: 8.0,
            usage: 8.0,
            delta: 1.0,
            buffered: false,
            capacity: 0.0,
            status: 0.0,
            cheating: false,
        });
        graph.batteries.push(PowerBattery {
            status: 0.5,
            capacity: 20.0,
            enabled: true,
        });

        graph.update_with_delta(2.0);

        assert_eq!(graph.last_power_needed, 8.0);
        assert_eq!(graph.last_power_produced, 8.0);
        assert_eq!(graph.last_power_stored, 10.0);
        assert_eq!(graph.last_capacity, 20.0);
        assert_eq!(graph.last_scaled_power_in, 2.0);
        assert_eq!(graph.last_scaled_power_out, 4.0);
        assert_eq!(graph.consumers[0].status, 1.0);
        assert!((graph.batteries[0].status - 0.3).abs() < 0.0001);
        assert_eq!(graph.power_balance(), -2.0);
        assert!(!graph.has_power_balance_samples());
    }

    #[test]
    fn power_graph_runtime_update_charges_batteries_and_handles_cheating_consumers() {
        let mut graph = PowerGraphRuntime::new();
        graph.producers.push(PowerProducer {
            production: 10.0,
            delta: 1.0,
        });
        graph.consumers.push(PowerConsumer {
            should_consume_power: true,
            requested_power: 4.0,
            usage: 4.0,
            delta: 1.0,
            buffered: true,
            capacity: 20.0,
            status: 0.25,
            cheating: false,
        });
        graph.batteries.push(PowerBattery {
            status: 0.25,
            capacity: 20.0,
            enabled: true,
        });

        graph.update_with_delta(1.0);

        assert_eq!(graph.last_power_needed, 4.0);
        assert_eq!(graph.last_power_produced, 10.0);
        assert!((graph.batteries[0].status - 0.55).abs() < 0.0001);
        assert!((graph.consumers[0].status - 0.45).abs() < 0.0001);

        graph.consumers[0].cheating = true;
        graph.update_with_delta(1.0);

        assert_eq!(graph.consumers[0].status, 1.0);
        assert_eq!(graph.last_power_needed, 1.0);
        assert_eq!(graph.last_power_produced, 1.0);
    }

    fn graph_node(id: i32) -> PowerGraphNode {
        PowerGraphNode {
            id,
            outputs_power: false,
            consumes_power: false,
            buffered: false,
            should_consume_power: true,
            production: 0.0,
            requested_power: 0.0,
            usage: 0.0,
            delta: 1.0,
            battery_status: 0.0,
            battery_capacity: 0.0,
            enabled: true,
            cheating: false,
        }
    }

    #[test]
    fn power_graph_runtime_add_node_classifies_and_clear_matches_java_lists() {
        let mut graph = PowerGraphRuntime::new();
        assert!(graph.add_node(PowerGraphNode {
            outputs_power: true,
            production: 3.0,
            ..graph_node(1)
        }));
        assert!(graph.add_node(PowerGraphNode {
            consumes_power: true,
            requested_power: 4.0,
            usage: 4.0,
            ..graph_node(2)
        }));
        assert!(graph.add_node(PowerGraphNode {
            outputs_power: true,
            consumes_power: true,
            buffered: true,
            battery_status: 0.25,
            battery_capacity: 20.0,
            ..graph_node(3)
        }));
        assert!(graph.add_node(PowerGraphNode {
            outputs_power: true,
            consumes_power: true,
            buffered: false,
            production: 2.0,
            requested_power: 1.0,
            usage: 1.0,
            ..graph_node(4)
        }));
        assert!(!graph.add_node(graph_node(4)));

        assert_eq!(graph.all, vec![1, 2, 3, 4]);
        assert_eq!(graph.producer_nodes, vec![1, 4]);
        assert_eq!(graph.consumer_nodes, vec![2, 4]);
        assert_eq!(graph.battery_nodes, vec![3]);

        assert!(graph.remove_list(4));
        assert_eq!(graph.all, vec![1, 2, 3]);
        assert_eq!(graph.producer_nodes, vec![1]);
        assert_eq!(graph.consumer_nodes, vec![2]);
        assert!(!graph.remove_list(99));

        graph.clear();
        assert!(graph.all.is_empty());
        assert!(graph.producers.is_empty());
        assert!(graph.consumers.is_empty());
        assert!(graph.batteries.is_empty());
    }

    #[test]
    fn power_graph_runtime_reflow_and_add_graph_follow_bfs_membership() {
        let mut graph = PowerGraphRuntime::new();
        graph.reflow_from(
            PowerGraphNode {
                outputs_power: true,
                ..graph_node(1)
            },
            |id| match id {
                1 => vec![
                    PowerGraphNode {
                        consumes_power: true,
                        requested_power: 2.0,
                        usage: 2.0,
                        ..graph_node(2)
                    },
                    PowerGraphNode {
                        outputs_power: true,
                        consumes_power: true,
                        buffered: true,
                        battery_capacity: 10.0,
                        ..graph_node(3)
                    },
                ],
                2 => vec![PowerGraphNode {
                    outputs_power: true,
                    consumes_power: true,
                    buffered: true,
                    battery_capacity: 10.0,
                    ..graph_node(3)
                }],
                _ => Vec::new(),
            },
        );

        assert_eq!(graph.all, vec![1, 2, 3]);
        assert_eq!(graph.producer_nodes, vec![1]);
        assert_eq!(graph.consumer_nodes, vec![2]);
        assert_eq!(graph.battery_nodes, vec![3]);

        let mut other = PowerGraphRuntime::new();
        other.add_node(PowerGraphNode {
            outputs_power: true,
            production: 5.0,
            ..graph_node(4)
        });
        graph.add_graph(&mut other);

        assert_eq!(graph.all, vec![1, 2, 3, 4]);
        assert!(other.all.is_empty());
        assert_eq!(graph.producer_nodes, vec![1, 4]);
    }

    #[test]
    fn power_graph_runtime_remove_with_connections_splits_branches_and_invalidates_old_graph() {
        let mut graph = PowerGraphRuntime::new();
        for node in [
            PowerGraphNode {
                outputs_power: true,
                production: 4.0,
                ..graph_node(1)
            },
            PowerGraphNode {
                consumes_power: true,
                requested_power: 2.0,
                usage: 2.0,
                ..graph_node(2)
            },
            PowerGraphNode {
                consumes_power: true,
                requested_power: 3.0,
                usage: 3.0,
                ..graph_node(3)
            },
            PowerGraphNode {
                outputs_power: true,
                production: 2.0,
                ..graph_node(4)
            },
        ] {
            graph.add_node(node);
        }

        let branches = graph.remove_with_connections(
            1,
            |id| match id {
                1 => vec![2, 3],
                2 => vec![1, 4],
                3 => vec![1],
                4 => vec![2],
                _ => Vec::new(),
            },
            |id| {
                Some(match id {
                    2 => PowerGraphNode {
                        consumes_power: true,
                        requested_power: 2.0,
                        usage: 2.0,
                        ..graph_node(2)
                    },
                    3 => PowerGraphNode {
                        consumes_power: true,
                        requested_power: 3.0,
                        usage: 3.0,
                        ..graph_node(3)
                    },
                    4 => PowerGraphNode {
                        outputs_power: true,
                        production: 2.0,
                        ..graph_node(4)
                    },
                    _ => return None,
                })
            },
        );

        assert!(graph.all.is_empty());
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].all, vec![2, 4]);
        assert_eq!(branches[0].consumer_nodes, vec![2]);
        assert_eq!(branches[0].producer_nodes, vec![4]);
        assert_eq!(branches[0].last_power_needed, 2.0);
        assert_eq!(branches[0].last_power_produced, 2.0);
        assert_eq!(branches[1].all, vec![3]);
        assert_eq!(branches[1].consumer_nodes, vec![3]);
        assert_eq!(branches[1].last_power_needed, 3.0);
        assert_eq!(branches[1].last_power_produced, 0.0);
    }
}
