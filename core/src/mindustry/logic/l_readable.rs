//! Mirrors upstream `mindustry.logic.LReadable`.

/// Logic object that can expose a value at a logic-memory-like position.
///
/// Java accepts the concrete `LExecutor` and `LVar` types. This Rust trait is
/// generic so lightweight runtime shells and tests can plug in equivalent
/// executor/variable representations while preserving the same contract.
pub trait LReadable<E, V> {
    fn readable(&self, exec: &E) -> bool;
    fn read(&self, position: &V, output: &mut V);
}

#[cfg(test)]
mod tests {
    use super::LReadable;

    #[derive(Default)]
    struct Exec {
        allowed: bool,
    }

    struct MemoryCell {
        value: f64,
    }

    impl LReadable<Exec, f64> for MemoryCell {
        fn readable(&self, exec: &Exec) -> bool {
            exec.allowed
        }

        fn read(&self, position: &f64, output: &mut f64) {
            *output = self.value + *position;
        }
    }

    #[test]
    fn l_readable_trait_exposes_java_readable_and_read_contract() {
        let readable = MemoryCell { value: 7.0 };
        assert!(readable.readable(&Exec { allowed: true }));
        assert!(!readable.readable(&Exec { allowed: false }));

        let mut output = 0.0;
        readable.read(&3.0, &mut output);
        assert_eq!(output, 10.0);
    }
}
