use std::io::{self, Read, Write};

pub trait BlockModule {
    fn write<W: Write>(&self, write: &mut W) -> io::Result<()>;

    fn read<R: Read>(&mut self, read: &mut R) -> io::Result<()>;

    fn read_legacy<R: Read>(&mut self, read: &mut R, _legacy: bool) -> io::Result<()> {
        self.read(read)
    }

    fn clear(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct CounterModule {
        value: u8,
        cleared: bool,
    }

    impl BlockModule for CounterModule {
        fn write<W: Write>(&self, write: &mut W) -> io::Result<()> {
            write.write_all(&[self.value])
        }

        fn read<R: Read>(&mut self, read: &mut R) -> io::Result<()> {
            let mut byte = [0];
            read.read_exact(&mut byte)?;
            self.value = byte[0];
            Ok(())
        }

        fn clear(&mut self) {
            self.cleared = true;
        }
    }

    #[test]
    fn block_module_write_and_default_legacy_read_match_java_contract_shape() {
        let module = CounterModule {
            value: 7,
            cleared: false,
        };
        let mut bytes = Vec::new();
        module.write(&mut bytes).unwrap();
        assert_eq!(bytes, vec![7]);

        let mut module = CounterModule::default();
        module.read_legacy(&mut bytes.as_slice(), true).unwrap();
        assert_eq!(module.value, 7);
    }

    #[test]
    fn clear_remains_optional_noop_hook_for_modules() {
        let mut module = CounterModule::default();
        module.clear();
        assert!(module.cleared);
    }
}
