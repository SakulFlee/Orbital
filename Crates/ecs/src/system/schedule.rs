use crate::World;
use crate::system::access::ComponentAccess;
use crate::system::commands::Commands;
use crate::system::executor::{Executor, SnapshotExecutor};
use crate::system::system::{IntoSystem, System, low_priority, with_interval};

/// A system plus the schedule-owned scheduling state.
struct Entry {
    system: Box<dyn System>,
    /// Schedule time (seconds, as passed to `run_with_time`) at which this
    /// system last executed. `None` until it has run at least once.
    last_run: Option<f64>,
}

impl Entry {
    /// Whether this system should execute at schedule time `now_secs`.
    ///
    /// - Systems without a `desired_interval` are always due (the fast path).
    /// - A throttled system with no previous run is due immediately (first tick).
    /// - Otherwise the full interval must have elapsed. Time that elapsed
    ///   while the system was skipped is *not* made up: it simply becomes
    ///   due once, on the next update cycle.
    fn is_due(&self, now_secs: f64) -> bool {
        let Some(interval) = self.system.desired_interval() else {
            return true;
        };
        match self.last_run {
            None => true,
            Some(last) => now_secs - last >= interval.as_secs_f64(),
        }
    }
}

pub struct Schedule {
    entries: Vec<Entry>,
    executor: Box<dyn Executor>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            executor: Box::new(SnapshotExecutor),
        }
    }

    pub fn with_executor(executor: Box<dyn Executor>) -> Self {
        Self {
            entries: Vec::new(),
            executor,
        }
    }

    pub fn add_system<M, S>(&mut self, system: S)
    where
        S: IntoSystem<M, System = Box<dyn System>>,
    {
        self.entries.push(Entry {
            system: system.into_system(),
            last_run: None,
        });
    }

    /// Add a system that runs at most once per `interval`, instead of every
    /// update cycle. See [`System::desired_interval`] for the exact rules
    /// (first tick always runs; at most one run per `run_with_time` call).
    ///
    /// Requires a clock: pass the current time via
    /// [`Schedule::run_with_time`]. Plain [`Schedule::run`] ignores
    /// intervals and executes everything (useful for tests).
    pub fn add_system_with_interval<M, S>(&mut self, system: S, interval: std::time::Duration)
    where
        S: IntoSystem<M, System = Box<dyn System>>,
    {
        let system = system.into_system();
        self.entries.push(Entry {
            system: with_interval(system, interval),
            last_run: None,
        });
    }

    /// Add a low-priority system: runs once per second instead of every
    /// update cycle. See [`LOW_PRIORITY_INTERVAL`](crate::LOW_PRIORITY_INTERVAL).
    pub fn add_low_priority_system<M, S>(&mut self, system: S)
    where
        S: IntoSystem<M, System = Box<dyn System>>,
    {
        let system = system.into_system();
        self.entries.push(Entry {
            system: low_priority(system),
            last_run: None,
        });
    }

    /// Add a pre-boxed system directly (useful for systems returned from
    /// Module::setup). If the boxed system reports a `desired_interval`
    /// (e.g. via [`crate::system::system::with_interval`]), it is throttled
    /// accordingly when the schedule is driven by [`Schedule::run_with_time`].
    pub fn add_system_boxed(&mut self, system: Box<dyn System>) {
        self.entries.push(Entry {
            system,
            last_run: None,
        });
    }

    /// Add a pre-boxed system that runs at most once per `interval`.
    pub fn add_system_boxed_with_interval(
        &mut self,
        system: Box<dyn System>,
        interval: std::time::Duration,
    ) {
        self.entries.push(Entry {
            system: with_interval(system, interval),
            last_run: None,
        });
    }

    /// Returns the number of systems in this schedule.
    pub fn system_count(&self) -> usize {
        self.entries.len()
    }

    /// Runs every system in this schedule, ignoring any desired intervals.
    ///
    /// This is the backwards-compatible behaviour and is convenient in
    /// tests. Frame loops should use [`Schedule::run_with_time`] so that
    /// low-priority (interval) systems are throttled against the clock.
    pub fn run(&mut self, world: &mut World) {
        self.execute(world, None);
    }

    /// Runs this schedule at schedule time `now_secs` (seconds, e.g.
    /// `TotalTime::0`), throttling systems that declare a desired interval.
    ///
    /// A throttled system runs when `now_secs - last_run >= interval`; it
    /// runs at most once per call, and always on its first call. Missed
    /// ticks are dropped, never replayed, so a long frame cannot trigger a
    /// burst of catch-up executions.
    pub fn run_with_time(&mut self, world: &mut World, now_secs: f64) {
        self.execute(world, Some(now_secs));
    }

    fn execute(&mut self, world: &mut World, now_secs: Option<f64>) {
        if self.entries.is_empty() {
            return;
        }

        // Select due systems. Without a clock everything is due (see `run`).
        let due: Vec<usize> = (0..self.entries.len())
            .filter(|&i| now_secs.is_none_or(|now| self.entries[i].is_due(now)))
            .collect();
        if due.is_empty() {
            return;
        }

        let batches = build_batches(&self.entries, &due);
        let mut commands = Commands::new();

        for indices in batches {
            // Collect unique mutable references to systems in this batch
            let batch_ptrs: Vec<*mut dyn System> = indices
                .iter()
                .map(|&i| self.entries[i].system.as_mut() as *mut dyn System)
                .collect();
            let mut refs: Vec<&mut dyn System> =
                batch_ptrs.iter().map(|&p| unsafe { &mut *p }).collect();
            self.executor.execute(&mut refs, world, &mut commands);
        }

        commands.flush(world).expect("Commands flush failed");

        // Record execution time for throttled systems. At most one run per
        // call happens (we stamp `now_secs`, not per-system elapsed time),
        // so a stalled frame results in a single run, not catch-up bursts.
        if let Some(now) = now_secs {
            for &i in &due {
                if self.entries[i].system.desired_interval().is_some() {
                    self.entries[i].last_run = Some(now);
                }
            }
        }
    }

    /// Greedy conflict batching over every system in this schedule,
    /// ignoring intervals. See [`build_batches`].
    pub fn build_batches(&self) -> Vec<Vec<usize>> {
        let all: Vec<usize> = (0..self.entries.len()).collect();
        build_batches(&self.entries, &all)
    }
}

/// Greedy conflict batching: place each system in the first batch it doesn't
/// conflict with, preserving registration order within the selected subset.
///
/// Only systems that are due this cycle should be passed in `subset`, so
/// skipped systems don't influence batching.
fn build_batches(entries: &[Entry], subset: &[usize]) -> Vec<Vec<usize>> {
    if subset.len() <= 1 {
        return subset.iter().map(|&i| vec![i]).collect();
    }

    let mut batches: Vec<Vec<usize>> = Vec::new();
    let mut batch_access: Vec<ComponentAccess> = Vec::new();

    for &idx in subset {
        let access = entries[idx].system.access();
        let mut placed = false;

        for (bi, existing) in batch_access.iter().enumerate() {
            if !access.conflicts_with(existing) {
                batches[bi].push(idx);
                let mut merged = existing.clone();
                merged.reads.extend(&access.reads);
                merged.writes.extend(&access.writes);
                batch_access[bi] = merged;
                placed = true;
                break;
            }
        }

        if !placed {
            batch_access.push(access.clone());
            batches.push(vec![idx]);
        }
    }

    batches
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::*;
    use crate::system::system::LOW_PRIORITY_INTERVAL;

    #[derive(Debug, Clone, Copy, Default)]
    #[allow(dead_code)]
    struct Pos(f32, f32);

    #[test]
    fn build_batches_no_conflict() {
        let mut schedule = Schedule::new();
        schedule.add_system::<fn(&Pos), _>(|_: &Pos| {});
        schedule.add_system::<fn(&Pos), _>(|_: &Pos| {});
        schedule.add_system::<fn(&Pos), _>(|_: &Pos| {});

        let batches = schedule.build_batches();
        // All three read-only systems should be in one batch
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 3);
    }

    #[test]
    fn build_batches_write_conflict() {
        let mut schedule = Schedule::new();
        schedule.add_system::<fn(&mut Pos), _>(|_: &mut Pos| {});
        schedule.add_system::<fn(&mut Pos), _>(|_: &mut Pos| {});

        let batches = schedule.build_batches();
        // Two write systems conflict — should be in separate batches
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn build_batches_mixed() {
        let mut schedule = Schedule::new();
        schedule.add_system::<fn(&Pos), _>(|_: &Pos| {}); // read
        schedule.add_system::<fn(&mut Pos), _>(|_: &mut Pos| {}); // write
        schedule.add_system::<fn(&Pos), _>(|_: &Pos| {}); // read

        let batches = schedule.build_batches();
        // read + write conflict, but read + read don't
        // Batch 0: read + read, Batch 1: write
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn add_system_boxed() {
        let mut schedule = Schedule::new();
        let system: Box<dyn crate::System> = (|_: &mut Pos| {}).into_system();
        schedule.add_system_boxed(system);
        assert_eq!(schedule.system_count(), 1);
    }

    #[test]
    fn system_count() {
        let mut schedule = Schedule::new();
        assert_eq!(schedule.system_count(), 0);
        schedule.add_system::<fn(&Pos), _>(|_: &Pos| {});
        assert_eq!(schedule.system_count(), 1);
        schedule.add_system::<fn(&Pos), _>(|_: &Pos| {});
        assert_eq!(schedule.system_count(), 2);
    }

    // -----------------------------------------------------------------------
    // Interval (low priority) scheduling
    // -----------------------------------------------------------------------

    fn counting_system(counter: Arc<AtomicUsize>) -> impl FnMut(&Pos) + Send + 'static {
        move |_: &Pos| {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn world_with_pos() -> World {
        let mut world = World::new();
        let e = world.spawn_entity();
        world.attach_component(&e, Pos(0.0, 0.0)).unwrap();
        world
    }

    #[test]
    fn interval_system_runs_on_first_tick() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut world = world_with_pos();
        let mut schedule = Schedule::new();
        schedule.add_system_with_interval(
            counting_system(Arc::clone(&counter)),
            Duration::from_secs(1),
        );

        schedule.run_with_time(&mut world, 0.0);
        assert_eq!(counter.load(Ordering::Relaxed), 1, "first tick must run");
    }

    #[test]
    fn interval_system_throttles_between_ticks() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut world = world_with_pos();
        let mut schedule = Schedule::new();
        schedule.add_system_with_interval(
            counting_system(Arc::clone(&counter)),
            Duration::from_secs(1),
        );

        schedule.run_with_time(&mut world, 0.0); // first tick: runs
        schedule.run_with_time(&mut world, 0.5); // 0.5s < 1s: skipped
        schedule.run_with_time(&mut world, 0.999); // still skipped
        schedule.run_with_time(&mut world, 1.0); // exactly 1s elapsed: runs
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn interval_system_drops_missed_ticks_no_catchup() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut world = world_with_pos();
        let mut schedule = Schedule::new();
        schedule.add_system_with_interval(
            counting_system(Arc::clone(&counter)),
            Duration::from_secs(1),
        );

        schedule.run_with_time(&mut world, 0.0); // runs
        schedule.run_with_time(&mut world, 5.0); // 5s stall: exactly one run
        assert_eq!(counter.load(Ordering::Relaxed), 2, "no catch-up burst");

        schedule.run_with_time(&mut world, 5.5); // 0.5s since last run: skipped
        assert_eq!(counter.load(Ordering::Relaxed), 2);
        schedule.run_with_time(&mut world, 6.0); // 1s since last run: runs
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn fast_system_runs_every_tick_alongside_throttled() {
        let fast = Arc::new(AtomicUsize::new(0));
        let slow = Arc::new(AtomicUsize::new(0));
        let mut world = world_with_pos();
        let mut schedule = Schedule::new();
        schedule.add_system::<fn(&Pos), _>(counting_system(Arc::clone(&fast)));
        schedule
            .add_system_with_interval(counting_system(Arc::clone(&slow)), Duration::from_secs(1));

        for now in [0.0, 0.25, 0.5, 0.75, 1.0] {
            schedule.run_with_time(&mut world, now);
        }
        assert_eq!(
            fast.load(Ordering::Relaxed),
            5,
            "fast system runs every tick"
        );
        assert_eq!(
            slow.load(Ordering::Relaxed),
            2,
            "slow system runs at 0.0 and 1.0"
        );
    }

    #[test]
    fn low_priority_system_is_one_hertz() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut world = world_with_pos();
        let mut schedule = Schedule::new();
        schedule.add_low_priority_system(counting_system(Arc::clone(&counter)));

        for now in 0..100 {
            schedule.run_with_time(&mut world, now as f64 * 0.02); // 50 FPS, 2s total
        }
        // t runs 0.00 … 1.98. Runs at t=0.0, then again at t>=1.0 (first tick
        // at t=1.00); t=2.00 is never reached, so exactly 2 runs.
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn run_ignores_intervals() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut world = world_with_pos();
        let mut schedule = Schedule::new();
        schedule.add_system_with_interval(
            counting_system(Arc::clone(&counter)),
            Duration::from_secs(60),
        );

        schedule.run(&mut world);
        schedule.run(&mut world);
        schedule.run(&mut world);
        assert_eq!(
            counter.load(Ordering::Relaxed),
            3,
            "plain run() must execute everything regardless of intervals"
        );
    }

    #[test]
    fn boxed_system_with_interval_is_throttled() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut world = world_with_pos();
        let mut schedule = Schedule::new();
        let boxed: Box<dyn crate::System> = counting_system(Arc::clone(&counter)).into_system();
        schedule.add_system_boxed_with_interval(boxed, Duration::from_secs(1));

        schedule.run_with_time(&mut world, 0.0); // runs
        schedule.run_with_time(&mut world, 0.5); // skipped
        schedule.run_with_time(&mut world, 1.0); // runs
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn with_interval_wrapper_reports_interval() {
        let boxed: Box<dyn crate::System> = (|_: &Pos| {}).into_system();
        assert_eq!(boxed.desired_interval(), None);
        let wrapped = with_interval(boxed, Duration::from_millis(250));
        assert_eq!(wrapped.desired_interval(), Some(Duration::from_millis(250)));
        let lp = low_priority((|_: &Pos| {}).into_system());
        assert_eq!(lp.desired_interval(), Some(LOW_PRIORITY_INTERVAL));
    }

    #[test]
    fn interval_system_skipped_cycles_do_not_flush_commands() {
        let mut world = World::new();
        let mut schedule = Schedule::new();
        // Throttled system that spawns an entity via Commands when it runs.
        schedule.add_system_with_interval(
            |cmds: &mut Commands| {
                let e = cmds.spawn_entity();
                cmds.attach_component(&e, 42i32);
            },
            Duration::from_secs(1),
        );

        schedule.run_with_time(&mut world, 0.0); // runs, spawns 1 entity
        schedule.run_with_time(&mut world, 0.5); // skipped — no spawn
        schedule.run_with_time(&mut world, 1.0); // runs, spawns 1 entity

        let store = world.get_component_store::<i32>().expect("store exists");
        assert_eq!(store.dense.len(), 2, "one spawn per due execution");
    }

    #[test]
    fn batching_uses_only_due_systems() {
        // A throttled writer and a fast writer conflict. On a tick where the
        // throttled one is skipped, the fast one should still run (batching
        // over the due subset only).
        let mut world = world_with_pos();
        let mut schedule = Schedule::new();
        schedule.add_system::<fn(&mut Pos), _>(|pos: &mut Pos| pos.0 += 100.0); // fast
        schedule.add_system_with_interval(|pos: &mut Pos| pos.0 += 1.0, Duration::from_secs(1)); // throttled

        schedule.run_with_time(&mut world, 0.0); // both run: pos = 101
        schedule.run_with_time(&mut world, 0.5); // fast only: pos = 201
        schedule.run_with_time(&mut world, 1.0); // both run: pos = 302

        let store = world.get_component_store::<Pos>().unwrap();
        let pos = store.get_component(0).unwrap();
        assert_eq!(pos.0, 302.0);
    }
}
