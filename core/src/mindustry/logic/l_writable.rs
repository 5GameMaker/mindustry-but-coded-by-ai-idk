//! Mirrors upstream `mindustry.logic.LWritable`.

/// Logic object that can write a value at a logic-memory-like position.
///
/// Java accepts concrete `LExecutor` and `LVar` parameters. This Rust trait is
/// generic for the same reason as `LReadable`: tests and runtime shells can
/// provide lightweight equivalent executor/variable representations.
pub trait LWritable<E, V> {
    fn writable(&self, exec: &E) -> bool;
    fn write(&mut self, position: &V, value: &V);
}

#[cfg(test)]
mod tests {
    use super::LWritable;

    #[derive(Default)]
    struct Exec {
        allowed: bool,
    }

    #[derive(Default)]
    struct MemoryCell {
        last_position: f64,
        last_value: f64,
    }

    impl LWritable<Exec, f64> for MemoryCell {
        fn writable(&self, exec: &Exec) -> bool {
            exec.allowed
        }

        fn write(&mut self, position: &f64, value: &f64) {
            self.last_position = *position;
            self.last_value = *value;
        }
    }

    #[test]
    fn l_writable_trait_exposes_java_writable_and_write_contract() {
        let mut writable = MemoryCell::default();
        assert!(writable.writable(&Exec { allowed: true }));
        assert!(!writable.writable(&Exec { allowed: false }));

        writable.write(&3.0, &9.0);
        assert_eq!(writable.last_position, 3.0);
        assert_eq!(writable.last_value, 9.0);
    }
}
