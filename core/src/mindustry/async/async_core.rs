use super::AsyncProcess;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncCoreBeginPlan {
    pub playing: bool,
    pub executor_size: Option<usize>,
    pub submitted: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncCore<P> {
    pub processes: Vec<P>,
    futures: Vec<usize>,
    executor_size: Option<usize>,
}

impl<P> AsyncCore<P>
where
    P: AsyncProcess,
{
    pub fn new(processes: Vec<P>) -> Self {
        Self {
            processes,
            futures: Vec::new(),
            executor_size: None,
        }
    }

    pub fn pending(&self) -> &[usize] {
        &self.futures
    }

    pub fn executor_size(&self) -> Option<usize> {
        self.executor_size
    }

    pub fn world_load(&mut self) {
        self.complete();
        for process in &mut self.processes {
            process.init();
        }
    }

    pub fn reset_event(&mut self) {
        self.complete();
        for process in &mut self.processes {
            process.reset();
        }
    }

    pub fn begin(&mut self, playing: bool) -> AsyncCoreBeginPlan {
        if !playing {
            return AsyncCoreBeginPlan {
                playing: false,
                executor_size: self.executor_size,
                submitted: Vec::new(),
            };
        }

        for process in &mut self.processes {
            process.begin();
        }

        self.futures.clear();
        let executor_size = *self.executor_size.get_or_insert(self.processes.len());

        for (index, process) in self.processes.iter().enumerate() {
            if process.should_process() {
                self.futures.push(index);
            }
        }

        AsyncCoreBeginPlan {
            playing: true,
            executor_size: Some(executor_size),
            submitted: self.futures.clone(),
        }
    }

    pub fn run_submitted(&mut self) {
        let submitted = self.futures.clone();
        for index in submitted {
            if let Some(process) = self.processes.get_mut(index) {
                process.process();
            }
        }
    }

    pub fn end(&mut self, playing: bool) {
        if playing {
            self.complete();
            for process in &mut self.processes {
                process.end();
            }
        }
    }

    pub fn complete(&mut self) {
        self.futures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordingProcess {
        name: &'static str,
        should_process: bool,
        calls: Vec<&'static str>,
    }

    impl RecordingProcess {
        fn new(name: &'static str, should_process: bool) -> Self {
            Self {
                name,
                should_process,
                calls: Vec::new(),
            }
        }
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
    fn async_core_world_load_and_reset_complete_pending_then_call_sync_hooks() {
        let mut core = AsyncCore::new(vec![RecordingProcess::new("physics", true)]);
        core.futures.push(0);

        core.world_load();
        assert!(core.pending().is_empty());
        assert_eq!(core.processes[0].calls, vec!["init"]);

        core.futures.push(0);
        core.reset_event();
        assert!(core.pending().is_empty());
        assert_eq!(core.processes[0].calls, vec!["init", "reset"]);
    }

    #[test]
    fn async_core_begin_submits_only_processes_that_should_process() {
        let mut core = AsyncCore::new(vec![
            RecordingProcess::new("physics", true),
            RecordingProcess::new("avoidance", false),
        ]);

        let plan = core.begin(true);
        assert_eq!(
            plan,
            AsyncCoreBeginPlan {
                playing: true,
                executor_size: Some(2),
                submitted: vec![0],
            }
        );
        assert_eq!(core.pending(), &[0]);
        assert_eq!(core.processes[0].calls, vec!["begin"]);
        assert_eq!(core.processes[1].calls, vec!["begin"]);

        core.run_submitted();
        assert_eq!(core.processes[0].calls, vec!["begin", "process"]);
        assert_eq!(core.processes[1].calls, vec!["begin"]);

        core.end(true);
        assert!(core.pending().is_empty());
        assert_eq!(core.processes[0].calls, vec!["begin", "process", "end"]);
        assert_eq!(core.processes[1].calls, vec!["begin", "end"]);
    }

    #[test]
    fn async_core_skips_lifecycle_when_state_is_not_playing() {
        let mut core = AsyncCore::new(vec![RecordingProcess::new("physics", true)]);

        let plan = core.begin(false);
        core.run_submitted();
        core.end(false);

        assert_eq!(
            plan,
            AsyncCoreBeginPlan {
                playing: false,
                executor_size: None,
                submitted: Vec::new(),
            }
        );
        assert!(core.processes[0].calls.is_empty());
        assert!(core.pending().is_empty());
    }
}
