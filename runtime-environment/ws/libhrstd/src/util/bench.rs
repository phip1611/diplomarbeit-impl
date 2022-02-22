use crate::time::{
    Duration,
    Instant,
};
use core::fmt::{
    Debug,
    Formatter,
};

pub type DurationPerIteration = Duration;

/// Helper script that benchmarks a workload [`BenchHelper::BENCH_ITERATIONS`] times.
/// Beforehand, it warms up the caches etc. with [`BenchHelper::WARMUP_ITERATIONS`] iterations.
#[derive(Debug)]
pub struct BenchHelper;

impl BenchHelper {
    const WARMUP_ITERATIONS: u64 = 10_000;
    const BENCH_ITERATIONS: u64 = 10_000;

    /// Performs warm-up iterations and executes the bench afterwards.
    /// Returns the duration per iteration.
    ///
    /// Consumes self so that captured mutable references get released.
    pub fn bench<F: FnMut(u64) -> ()>(mut fnc: F) -> DurationPerIteration {
        (0..Self::WARMUP_ITERATIONS).for_each(|i| fnc(i));
        let begin = Instant::now();
        (0..Self::BENCH_ITERATIONS).for_each(|i| fnc(i));
        (Instant::now() - begin) / Self::BENCH_ITERATIONS
    }
}

