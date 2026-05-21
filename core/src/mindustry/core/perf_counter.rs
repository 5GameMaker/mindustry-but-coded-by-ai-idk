use std::collections::VecDeque;
use std::fmt;
use std::sync::OnceLock;
use std::time::Instant;

/// Rust counterpart of the Java `PerfCounter` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum PerfCounterKind {
    Frame = 0,
    Update = 1,
    EntityUpdate = 2,
    Ui = 3,
    Render = 4,
}

impl PerfCounterKind {
    /// Matches the Java `values()` order.
    pub const ALL: [Self; 5] = [
        Self::Frame,
        Self::Update,
        Self::EntityUpdate,
        Self::Ui,
        Self::Render,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Frame => "frame",
            Self::Update => "update",
            Self::EntityUpdate => "entityUpdate",
            Self::Ui => "ui",
            Self::Render => "render",
        }
    }
}

impl fmt::Display for PerfCounterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub const PERF_COUNTER_MEAN_WINDOW: usize = 30;
pub const PERF_COUNTER_REFRESH_TIME_MILLIS: u64 = 500;
pub const NANOSECONDS_PER_MILLISECOND: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq)]
struct WindowedMean {
    window: usize,
    values: VecDeque<u64>,
    sum: u128,
}

impl WindowedMean {
    fn new(window: usize) -> Self {
        assert!(window > 0, "window size must be positive");
        Self {
            window,
            values: VecDeque::with_capacity(window),
            sum: 0,
        }
    }

    fn add(&mut self, value: u64) {
        if self.values.len() == self.window {
            if let Some(removed) = self.values.pop_front() {
                self.sum = self.sum.saturating_sub(removed as u128);
            }
        }
        self.values.push_back(value);
        self.sum += value as u128;
    }

    fn raw_mean(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.sum as f64 / self.values.len() as f64
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerfCounter {
    kind: PerfCounterKind,
    began: bool,
    begin_time_ns: u64,
    value_refresh_time_ms: u64,
    refresh_value_ms: f32,
    mean: WindowedMean,
}

impl PerfCounter {
    pub fn new(kind: PerfCounterKind) -> Self {
        Self {
            kind,
            began: false,
            begin_time_ns: 0,
            value_refresh_time_ms: 0,
            refresh_value_ms: 0.0,
            mean: WindowedMean::new(PERF_COUNTER_MEAN_WINDOW),
        }
    }

    pub fn kind(&self) -> PerfCounterKind {
        self.kind
    }

    pub fn is_began(&self) -> bool {
        self.began
    }

    pub fn sample_count(&self) -> usize {
        self.mean.len()
    }

    pub fn begin(&mut self) {
        self.begin_at(current_nanos());
    }

    pub fn begin_at(&mut self, begin_time_ns: u64) {
        self.began = true;
        self.begin_time_ns = begin_time_ns;
    }

    pub fn end(&mut self) {
        self.end_at(current_nanos());
    }

    pub fn end_at(&mut self, end_time_ns: u64) {
        if !self.began {
            return;
        }

        self.began = false;
        let elapsed_ns = end_time_ns.saturating_sub(self.begin_time_ns);
        self.mean.add(elapsed_ns);
    }

    /// Value with a periodic refresh interval applied, to prevent jittery UI.
    pub fn value_ms(&mut self) -> f32 {
        self.value_ms_at(current_millis())
    }

    pub fn value_ms_at(&mut self, now_millis: u64) -> f32 {
        if now_millis.saturating_sub(self.value_refresh_time_ms) > PERF_COUNTER_REFRESH_TIME_MILLIS
        {
            self.refresh_value_ms = self.raw_value_ms();
            self.value_refresh_time_ms = now_millis;
        }
        self.refresh_value_ms
    }

    /// Raw value without a refresh interval. This will be unstable.
    pub fn raw_value_ms(&self) -> f32 {
        self.mean.raw_mean() as f32 / NANOSECONDS_PER_MILLISECOND as f32
    }

    pub fn raw_value_ns(&self) -> u64 {
        self.mean.raw_mean() as u64
    }

    pub fn refresh_value_ms(&self) -> f32 {
        self.refresh_value_ms
    }

    pub fn value_refresh_time_ms(&self) -> u64 {
        self.value_refresh_time_ms
    }
}

impl Default for PerfCounter {
    fn default() -> Self {
        Self::new(PerfCounterKind::Frame)
    }
}

fn clock_origin() -> &'static Instant {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now)
}

fn current_nanos() -> u64 {
    clock_origin().elapsed().as_nanos().min(u64::MAX as u128) as u64
}

fn current_millis() -> u64 {
    clock_origin().elapsed().as_millis().min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record_duration_ns(counter: &mut PerfCounter, duration_ns: u64) {
        counter.begin_at(0);
        counter.end_at(duration_ns);
    }

    #[test]
    fn perf_counter_kind_matches_java_enum_order_and_labels() {
        assert_eq!(
            PerfCounterKind::ALL,
            [
                PerfCounterKind::Frame,
                PerfCounterKind::Update,
                PerfCounterKind::EntityUpdate,
                PerfCounterKind::Ui,
                PerfCounterKind::Render,
            ]
        );
        assert_eq!(PerfCounterKind::Frame.index(), 0);
        assert_eq!(PerfCounterKind::Render.index(), 4);
        assert_eq!(PerfCounterKind::EntityUpdate.as_str(), "entityUpdate");
        assert_eq!(PerfCounterKind::Ui.to_string(), "ui");
    }

    #[test]
    fn perf_counter_begin_end_ignores_unmatched_end_and_tracks_latest_begin() {
        let mut counter = PerfCounter::new(PerfCounterKind::Update);

        counter.end_at(1_000);
        assert_eq!(counter.sample_count(), 0);
        assert!(!counter.is_began());

        counter.begin_at(100);
        counter.begin_at(150);
        assert!(counter.is_began());

        counter.end_at(200);
        assert!(!counter.is_began());
        assert_eq!(counter.sample_count(), 1);
        assert_eq!(counter.raw_value_ns(), 50);
        assert!((counter.raw_value_ms() - 0.000_05).abs() < 1e-8);

        counter.end_at(250);
        assert_eq!(counter.sample_count(), 1);
        assert_eq!(counter.raw_value_ns(), 50);
    }

    #[test]
    fn perf_counter_window_keeps_only_last_thirty_samples() {
        let mut counter = PerfCounter::new(PerfCounterKind::Frame);

        for _ in 0..PERF_COUNTER_MEAN_WINDOW {
            record_duration_ns(&mut counter, 10 * NANOSECONDS_PER_MILLISECOND);
        }

        assert_eq!(counter.sample_count(), PERF_COUNTER_MEAN_WINDOW);
        assert_eq!(counter.raw_value_ns(), 10 * NANOSECONDS_PER_MILLISECOND);
        assert_eq!(counter.raw_value_ms(), 10.0);

        record_duration_ns(&mut counter, 40 * NANOSECONDS_PER_MILLISECOND);

        assert_eq!(counter.sample_count(), PERF_COUNTER_MEAN_WINDOW);
        assert_eq!(counter.raw_value_ns(), 11 * NANOSECONDS_PER_MILLISECOND);
        assert_eq!(counter.raw_value_ms(), 11.0);
    }

    #[test]
    fn perf_counter_value_ms_refreshes_only_after_five_hundred_milliseconds() {
        let mut counter = PerfCounter::new(PerfCounterKind::Ui);
        record_duration_ns(&mut counter, 20 * NANOSECONDS_PER_MILLISECOND);

        assert_eq!(counter.value_ms_at(0), 0.0);
        assert_eq!(counter.value_ms_at(PERF_COUNTER_REFRESH_TIME_MILLIS), 0.0);
        assert_eq!(
            counter.value_ms_at(PERF_COUNTER_REFRESH_TIME_MILLIS + 1),
            20.0
        );

        record_duration_ns(&mut counter, 40 * NANOSECONDS_PER_MILLISECOND);

        assert_eq!(counter.value_ms_at(750), 20.0);
        assert_eq!(counter.value_ms_at(1_002), 30.0);
        assert_eq!(counter.refresh_value_ms(), 30.0);
        assert_eq!(counter.value_refresh_time_ms(), 1_002);
    }
}
