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
pub struct BenchHelper<
    'a,
    BenchFncT: FnMut(u64) -> (),
    const WARMUP_ITERATIONS: u64 = 10_000,
    const BENCH_ITERATIONS: u64 = 10_000,
> {
    before_each_fn: Option<&'a mut dyn FnMut()>,
    bench_fn: BenchFncT,
    after_each_fn: Option<&'a mut dyn FnMut()>,
}

impl<
        'a,
        BenchFncT: FnMut(u64) -> (),
        const WARMUP_ITERATIONS: u64,
        const BENCH_ITERATIONS: u64,
    > BenchHelper<'a, BenchFncT, WARMUP_ITERATIONS, BENCH_ITERATIONS>
{
    /// Constructor.
    pub fn new(bench_fn: BenchFncT) -> Self {
        Self {
            before_each_fn: None,
            bench_fn,
            after_each_fn: None,
        }
    }

    /// Attaches a before each hook. Executed before each benchmark iteration
    /// but does not count onto the time.
    pub fn with_before_each(&mut self, before_each_fn: &'a mut dyn FnMut()) -> &mut Self {
        self.before_each_fn.replace(before_each_fn);
        self
    }

    /// Attaches a after each hook. Executed after each benchmark iteration
    /// but does not count onto the time.
    pub fn with_after_each(&mut self, after_each_fn: &'a mut dyn FnMut()) -> &mut Self {
        self.after_each_fn.replace(after_each_fn);
        self
    }

    /// Execute the benchmark and invokes the before_each and after_each
    /// callback in each iteration accordingly. The time for these hooks
    /// does not count onto the  time of the benchmark.
    pub fn bench(&mut self) -> DurationPerIteration {
        let mut counter = 0;
        // A single step of the benchmark. Executes the before_each callback if it is
        // provided. Performs the actual bench. Executes the after_each callback if it
        // is provided.
        //
        // Only measures the costs of the bench function isolated.
        let mut single_bench_round = |counter: &mut u64, iteration: u64| {
            if let Some(fnc) = self.before_each_fn.as_mut() {
                fnc();
            }
            let begin = Instant::now();
            (self.bench_fn)(iteration);
            *counter += Instant::now() - begin;
            if let Some(fnc) = self.after_each_fn.as_mut() {
                fnc();
            }
        };

        (0..WARMUP_ITERATIONS).for_each(|i| single_bench_round(&mut counter, i));
        counter = 0;
        (0..BENCH_ITERATIONS).for_each(|i| single_bench_round(&mut counter, i));
        counter / BENCH_ITERATIONS
    }

    /// Direct benchmark the function. For a more complex use with
    /// "before_each" and "after_each" hooks, please check [`Self::bench`].
    ///
    /// Performs warm-up iterations and executes the bench afterwards.
    /// Returns the duration per iteration.
    ///
    /// # Example
    /// ```ignore
    /// // specify: 2 warmup rounds, 3 bench rounds
    /// BenchHelper::<_, 2, 3>::new(|i| println!("Bench Iteration #{}", i)).bench();
    /// BenchHelper::<_>::new(|i| println!("Bench Iteration #{}", i)).bench();
    /// ```
    pub fn bench_direct(mut fnc: BenchFncT) -> DurationPerIteration {
        (0..WARMUP_ITERATIONS).for_each(|i| fnc(i));
        let begin = Instant::now();
        (0..BENCH_ITERATIONS).for_each(|i| fnc(i));
        (Instant::now() - begin) / BENCH_ITERATIONS
    }
}

impl<
        'a,
        BenchFncT: FnMut(u64) -> (),
        const WARMUP_ITERATIONS: u64,
        const BENCH_ITERATIONS: u64,
    > Debug for BenchHelper<'a, BenchFncT, WARMUP_ITERATIONS, BENCH_ITERATIONS>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BenchHelper")
            .field("warmup_iterations", &WARMUP_ITERATIONS)
            .field("bench_iterations", &BENCH_ITERATIONS)
            .field(
                "before_each_hook",
                &if self.before_each_fn.is_some() {
                    "<present>"
                } else {
                    "<none>"
                },
            )
            .field(
                "after_each_hook",
                &if self.after_each_fn.is_some() {
                    "<present>"
                } else {
                    "<none>"
                },
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::time::Instant;
    use crate::util::BenchHelper;
    use std::println;

    #[test]
    fn test_bench_direct() {
        let mut x = 0;
        let _ = BenchHelper::<_, 1, 2>::bench_direct(|_| x += 1);
    }

    #[test]
    fn test_bench_with_hooks() {
        let mut before_each_hook = || println!("I'm the before hook!");
        let mut after_each_hook = || println!("I'm the after hook!");
        let mut bench = BenchHelper::<_, 2, 3>::new(|i| println!("Bench Iteration #{}", i));
        bench
            .with_before_each(&mut before_each_hook)
            .with_after_each(&mut after_each_hook);
        let begin = Instant::now();
        let res = bench.bench();
        let end = Instant::now();
        assert!(
            res < end - begin,
            "the overhead of the before and after callback must be noticeable"
        );
        println!("took {} ticks", res);
    }

    #[test]
    fn test_bench_without_hooks() {
        let _ = BenchHelper::<_, 2, 3>::new(|i| println!("Bench Iteration #{}", i)).bench();
    }

    #[test]
    fn test_bench_const_generic_infer() {
        let mut counter = 0;
        let _ = BenchHelper::<_>::new(|i| counter = i).bench();
        assert_eq!(counter, 10000 - 1);
    }
}
