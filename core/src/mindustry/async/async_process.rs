//! Async process hook trait mirroring upstream `mindustry.async.AsyncProcess`.

pub trait AsyncProcess {
    /// Sync. Called when the world loads.
    fn init(&mut self) {}

    /// Sync. Called when the world resets.
    fn reset(&mut self) {}

    /// Sync. Called at the beginning of the main loop.
    fn begin(&mut self) {}

    /// Async. Called in a separate thread.
    fn process(&mut self) {}

    /// Sync. Called in the end of the main loop.
    fn end(&mut self) {}

    fn should_process(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct DefaultHooks;

    impl AsyncProcess for DefaultHooks {}

    #[derive(Default)]
    struct RecordingProcess {
        calls: Vec<&'static str>,
        should_process: bool,
    }

    impl AsyncProcess for RecordingProcess {
        fn init(&mut self) {
            self.calls.push("init");
        }

        fn reset(&mut self) {
            self.calls.push("reset");
        }

        fn begin(&mut self) {
            self.calls.push("begin");
        }

        fn process(&mut self) {
            self.calls.push("process");
        }

        fn end(&mut self) {
            self.calls.push("end");
        }

        fn should_process(&self) -> bool {
            self.should_process
        }
    }

    #[test]
    fn default_hooks_are_noop_and_should_process_is_true_like_java() {
        let mut process = DefaultHooks;

        process.init();
        process.reset();
        process.begin();
        process.process();
        process.end();

        assert!(process.should_process());
    }

    #[test]
    fn process_trait_allows_backends_to_record_java_hook_order() {
        let mut process = RecordingProcess {
            should_process: false,
            ..Default::default()
        };

        process.init();
        process.reset();
        process.begin();
        process.process();
        process.end();

        assert_eq!(
            process.calls,
            vec!["init", "reset", "begin", "process", "end"]
        );
        assert!(!process.should_process());
    }
}
