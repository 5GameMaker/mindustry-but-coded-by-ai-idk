//! Mirrors the runtime executor container around upstream `LExecutor`.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    assemble_logic_source, LVar, LogicAssembler, LogicBulletEvent, LogicClientDataEvent,
    LogicCutsceneState, LogicEffectEvent, LogicExplosionEvent, LogicInstruction, LogicMarkerEvent,
    LogicMarkerObject, LogicMessageEvent, LogicMessageState, LogicParseError, LogicRulesState,
    LogicRuntimeObject, LogicSoundEvent, LogicSpawnEvent, LogicSyncEvent, LogicWeatherEvent,
    LogicWeatherState, LogicWorldObject, RadarUnitView, LOGIC_DEFAULT_MAX_IPT,
    LOGIC_SYNC_INTERVAL_MILLIS,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LogicExecutor {
    pub instructions: Vec<LogicInstruction>,
    pub vars: Vec<LVar>,
    pub counter: LVar,
    pub yield_: bool,
    pub privileged: bool,
    pub team: u8,
    pub links: Vec<String>,
    pub objects: BTreeMap<String, LogicRuntimeObject>,
    pub radar_units: BTreeMap<String, RadarUnitView>,
    pub world: LogicWorldObject,
    pub query_result: Option<String>,
    pub objective_flags: BTreeSet<String>,
    pub rules: LogicRulesState,
    pub is_client: bool,
    pub ipt: i32,
    pub max_ipt: i32,
    pub current_time_millis: i64,
    pub spawn_events: Vec<LogicSpawnEvent>,
    pub bullet_events: Vec<LogicBulletEvent>,
    pub effect_events: Vec<LogicEffectEvent>,
    pub explosion_events: Vec<LogicExplosionEvent>,
    pub weather_states: BTreeMap<String, LogicWeatherState>,
    pub weather_events: Vec<LogicWeatherEvent>,
    pub message_events: Vec<LogicMessageEvent>,
    pub client_data_events: Vec<LogicClientDataEvent>,
    pub sync_events: Vec<LogicSyncEvent>,
    pub sound_events: Vec<LogicSoundEvent>,
    pub markers: BTreeMap<i32, LogicMarkerObject>,
    pub marker_events: Vec<LogicMarkerEvent>,
    pub map_locales: BTreeMap<String, String>,
    pub mobile: bool,
    pub message_state: LogicMessageState,
    pub cutscene: LogicCutsceneState,
    pub spawn_wave_events: Vec<(f32, f32, bool)>,
    pub bound_unit: Option<String>,
    pub unit_binds: BTreeMap<String, usize>,
    pub logic_unit_control: bool,
    pub headless: bool,
    pub graphics_buffer: Vec<u64>,
    pub text_buffer: String,
}

impl Default for LogicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicExecutor {
    pub const MAX_TEXT_BUFFER: usize = 400;
    pub const MAX_GRAPHICS_BUFFER: usize = 256;

    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            vars: Vec::new(),
            counter: {
                let mut counter = LVar::new("@counter");
                counter.is_obj = false;
                counter
            },
            yield_: false,
            privileged: false,
            team: 0,
            links: Vec::new(),
            objects: BTreeMap::new(),
            radar_units: BTreeMap::new(),
            world: LogicWorldObject::new(),
            query_result: Some("@query".into()),
            objective_flags: BTreeSet::new(),
            rules: LogicRulesState::default(),
            is_client: false,
            ipt: 1,
            max_ipt: LOGIC_DEFAULT_MAX_IPT,
            current_time_millis: LOGIC_SYNC_INTERVAL_MILLIS,
            spawn_events: Vec::new(),
            bullet_events: Vec::new(),
            effect_events: Vec::new(),
            explosion_events: Vec::new(),
            weather_states: BTreeMap::new(),
            weather_events: Vec::new(),
            message_events: Vec::new(),
            client_data_events: Vec::new(),
            sync_events: Vec::new(),
            sound_events: Vec::new(),
            markers: BTreeMap::new(),
            marker_events: Vec::new(),
            map_locales: BTreeMap::new(),
            mobile: false,
            message_state: LogicMessageState::default(),
            cutscene: LogicCutsceneState::default(),
            spawn_wave_events: Vec::new(),
            bound_unit: None,
            unit_binds: BTreeMap::new(),
            logic_unit_control: true,
            headless: false,
            graphics_buffer: Vec::new(),
            text_buffer: String::new(),
        }
    }

    pub fn from_source(source: &str, privileged: bool) -> Result<Self, LogicParseError> {
        let (assembler, instructions) = assemble_logic_source(source, privileged)?;
        let mut exec = Self::new();
        exec.load_assembled(assembler, instructions);
        Ok(exec)
    }

    pub fn load_assembled(
        &mut self,
        assembler: LogicAssembler,
        instructions: Vec<LogicInstruction>,
    ) {
        self.privileged = assembler.privileged;
        self.vars = assembler
            .vars
            .into_values()
            .filter(|var| !var.constant)
            .collect();
        for (id, var) in self.vars.iter_mut().enumerate() {
            var.id = id as i32;
        }
        if let Some(counter) = self.vars.iter().find(|var| var.name == "@counter") {
            self.counter = counter.clone();
        }
        self.instructions = instructions;
        self.sync_instructions_from_vars();
    }

    pub fn run_steps(&mut self, max_steps: usize) -> usize {
        let mut steps = 0;
        while steps < max_steps
            && !self.yield_
            && self.counter.numval >= 0.0
            && self.counter.numval < self.instructions.len() as f64
        {
            self.run_once();
            steps += 1;
        }
        steps
    }

    pub fn var_by_name(&self, name: &str) -> Option<&LVar> {
        if name == "@counter" {
            return Some(&self.counter);
        }
        self.vars.iter().find(|var| var.name == name)
    }

    pub fn var_by_name_mut(&mut self, name: &str) -> Option<&mut LVar> {
        if name == "@counter" {
            return Some(&mut self.counter);
        }
        self.vars.iter_mut().find(|var| var.name == name)
    }

    pub(super) fn upsert_runtime_var(&mut self, var: &LVar) {
        if var.name == "@counter" {
            self.counter = var.clone();
        }

        if let Some(shared) = self.vars.iter_mut().find(|shared| shared.name == var.name) {
            *shared = var.clone();
        } else {
            let id = self.vars.len() as i32;
            self.vars.push(var.clone());
            if let Some(last) = self.vars.last_mut() {
                last.id = id;
            }
        }
    }

    fn sync_instructions_from_vars(&mut self) {
        let vars = self.vars.clone();
        let counter = self.counter.clone();
        for instruction in &mut self.instructions {
            instruction.load_shared_vars_from(&vars, &counter);
        }
    }

    pub fn run_once(&mut self) {
        if self.counter.numval >= self.instructions.len() as f64 || self.counter.numval < 0.0 {
            self.counter.numval = 0.0;
        }

        if self.counter.numval < self.instructions.len() as f64 {
            self.counter.is_obj = false;
            let index = self.counter.numval as usize;
            self.counter.numval += 1.0;
            let mut instruction = self.instructions[index].clone();
            instruction.load_shared_vars(self);
            instruction.run(self);
            instruction.store_shared_vars(self);
            self.instructions[index] = instruction;
        }
    }

    pub fn push_text_bounded(&mut self, value: &str) {
        if self.text_buffer.len() >= Self::MAX_TEXT_BUFFER {
            return;
        }

        let remaining = Self::MAX_TEXT_BUFFER - self.text_buffer.len();
        if value.len() <= remaining {
            self.text_buffer.push_str(value);
            return;
        }

        let mut end = remaining;
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        self.text_buffer.push_str(&value[..end]);
    }

    pub fn register_object(&mut self, name: impl Into<String>, object: LogicRuntimeObject) {
        self.objects.insert(name.into(), object);
    }

    pub fn register_radar_unit(&mut self, name: impl Into<String>, unit: RadarUnitView) {
        self.radar_units.insert(name.into(), unit);
    }
}

#[cfg(test)]
mod tests {
    use super::LogicExecutor;
    use crate::mindustry::logic::{
        LVar, LogicRuntimeObject, RadarUnitView, LOGIC_DEFAULT_MAX_IPT, LOGIC_SYNC_INTERVAL_MILLIS,
    };

    #[test]
    fn logic_executor_defaults_match_java_l_executor_runtime_baseline() {
        let exec = LogicExecutor::new();
        assert!(exec.instructions.is_empty());
        assert!(exec.vars.is_empty());
        assert_eq!(exec.counter.name, "@counter");
        assert!(!exec.counter.is_obj);
        assert!(!exec.yield_);
        assert!(!exec.privileged);
        assert_eq!(exec.team, 0);
        assert!(exec.links.is_empty());
        assert!(exec.objects.is_empty());
        assert!(exec.radar_units.is_empty());
        assert_eq!(exec.query_result.as_deref(), Some("@query"));
        assert!(exec.objective_flags.is_empty());
        assert!(!exec.is_client);
        assert_eq!(exec.ipt, 1);
        assert_eq!(exec.max_ipt, LOGIC_DEFAULT_MAX_IPT);
        assert_eq!(exec.current_time_millis, LOGIC_SYNC_INTERVAL_MILLIS);
        assert!(exec.spawn_events.is_empty());
        assert!(exec.message_events.is_empty());
        assert!(!exec.mobile);
        assert!(exec.logic_unit_control);
        assert!(!exec.headless);
        assert!(exec.graphics_buffer.is_empty());
        assert!(exec.text_buffer.is_empty());
    }

    #[test]
    fn logic_executor_registers_objects_vars_and_bounds_text_like_runtime() {
        let mut exec = LogicExecutor::new();
        exec.register_object("text", LogicRuntimeObject::Text("hello".into()));
        exec.register_radar_unit("enemy", RadarUnitView::new(1.0, 2.0, 3));
        assert!(
            matches!(exec.objects.get("text"), Some(LogicRuntimeObject::Text(value)) if value == "hello")
        );
        assert_eq!(exec.radar_units["enemy"].team, 3);

        let mut var = LVar::new("value");
        var.set_num(7.0);
        exec.upsert_runtime_var(&var);
        assert_eq!(exec.var_by_name("value").unwrap().num(), 7.0);
        var.set_num(8.0);
        exec.upsert_runtime_var(&var);
        assert_eq!(exec.var_by_name("value").unwrap().num(), 8.0);

        let emoji = "💥";
        for _ in 0..LogicExecutor::MAX_TEXT_BUFFER {
            exec.push_text_bounded(emoji);
        }
        assert!(exec.text_buffer.len() <= LogicExecutor::MAX_TEXT_BUFFER);
        assert!(exec.text_buffer.is_char_boundary(exec.text_buffer.len()));
    }
}
