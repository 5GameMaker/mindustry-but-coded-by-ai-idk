//! Mirrors the memory-bank object contract used by upstream `LExecutor`.

use super::LVar;

#[derive(Debug, Clone, PartialEq)]
pub struct LogicMemoryObject {
    pub memory: Vec<f64>,
    pub team: u8,
    pub block_privileged: bool,
    pub valid: bool,
}

impl LogicMemoryObject {
    pub fn new(capacity: usize, team: u8) -> Self {
        Self {
            memory: vec![0.0; capacity],
            team,
            block_privileged: false,
            valid: true,
        }
    }

    pub fn readable_by(&self, exec_privileged: bool, exec_team: u8) -> bool {
        self.valid && (exec_privileged || (self.team == exec_team && !self.block_privileged))
    }

    pub fn read(&self, position: &LVar, output: &mut LVar) {
        let address = position.numi();
        if address < 0 || address as usize >= self.memory.len() {
            output.set_num(f64::NAN);
        } else {
            output.set_num(self.memory[address as usize]);
        }
    }

    pub fn write(&mut self, position: &LVar, value: &LVar) {
        let address = position.numi();
        if address < 0 || address as usize >= self.memory.len() {
            return;
        }
        self.memory[address as usize] = value.num();
    }
}

#[cfg(test)]
mod tests {
    use super::LogicMemoryObject;
    use crate::mindustry::logic::LVar;

    #[test]
    fn logic_memory_read_write_matches_java_bounds_and_privilege_contract() {
        let mut memory = LogicMemoryObject::new(2, 3);
        assert!(memory.readable_by(false, 3));
        assert!(!memory.readable_by(false, 2));
        assert!(memory.readable_by(true, 2));
        memory.block_privileged = true;
        assert!(!memory.readable_by(false, 3));
        memory.block_privileged = false;

        let mut position = LVar::new("pos");
        position.set_num(1.0);
        let mut value = LVar::new("value");
        value.set_num(42.0);
        memory.write(&position, &value);

        let mut output = LVar::new("out");
        memory.read(&position, &mut output);
        assert_eq!(output.num(), 42.0);

        position.set_num(-1.0);
        memory.write(&position, &value);
        memory.read(&position, &mut output);
        assert!(output.num_or_nan().is_nan());
    }
}
