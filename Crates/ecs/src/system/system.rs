use std::time::Duration;

use crate::system::access::ComponentAccess;
use crate::system::commands::Commands;

/// The preset used by "low priority" helpers: run at most once per second.
pub const LOW_PRIORITY_INTERVAL: Duration = Duration::from_secs(1);

pub trait System: Send {
    fn name(&self) -> &str;
    fn access(&self) -> &ComponentAccess;
    fn run(&mut self, world: &crate::World, commands: &mut Commands);

    /// How often this system wants to run, in wall-clock-ish schedule time
    /// (seconds passed to `Schedule::run_with_time`).
    ///
    /// `None` (the default) means "every update cycle" — the normal fast
    /// path. `Some(interval)` marks the system as low priority: the
    /// `Schedule` will only execute it once the interval has elapsed since
    /// its previous execution. The first run always happens immediately
    /// (the system has no previous execution yet), and a system is never
    /// run more than once per update cycle, even after a long stall —
    /// missed ticks are dropped rather than replayed.
    ///
    /// The `Schedule` owns the bookkeeping for this: systems themselves
    /// only declare their desired interval.
    fn desired_interval(&self) -> Option<std::time::Duration> {
        None
    }
}

pub struct FunctionSystemMetadata {
    pub name: &'static str,
    pub access: ComponentAccess,
}

pub struct FunctionSystem {
    pub metadata: FunctionSystemMetadata,
    #[allow(clippy::type_complexity)]
    run_fn: Box<dyn FnMut(&crate::World, &mut Commands) + Send>,
}

impl FunctionSystem {
    #[allow(clippy::type_complexity)]
    pub fn new(
        metadata: FunctionSystemMetadata,
        run_fn: Box<dyn FnMut(&crate::World, &mut Commands) + Send>,
    ) -> Self {
        Self { metadata, run_fn }
    }
}

impl System for FunctionSystem {
    fn name(&self) -> &str {
        self.metadata.name
    }
    fn access(&self) -> &ComponentAccess {
        &self.metadata.access
    }
    fn run(&mut self, world: &crate::World, commands: &mut Commands) {
        (self.run_fn)(world, commands);
    }
}

pub trait IntoSystem<Marker>: Sized {
    type System: System + 'static;
    fn into_system(self) -> Self::System;
}

/// Marks a system as running at a fixed interval (low priority scheduling).
///
/// Wraps an existing boxed system and reports `interval` from
/// [`System::desired_interval`], which the [`crate::Schedule`] consults to
/// decide whether the system is due this update cycle. The wrapper adds no
/// state of its own — the schedule owns the bookkeeping.
pub struct WithInterval {
    inner: Box<dyn System>,
    interval: Duration,
}

impl System for WithInterval {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn access(&self) -> &ComponentAccess {
        self.inner.access()
    }
    fn run(&mut self, world: &crate::World, commands: &mut Commands) {
        self.inner.run(world, commands)
    }
    fn desired_interval(&self) -> Option<Duration> {
        Some(self.interval)
    }
}

/// Wrap a boxed system so it runs at most once per `interval`.
///
/// Intended for systems returned from `Module::setup`, which are added via
/// `Schedule::add_system_boxed` and thus cannot use
/// `Schedule::add_system_with_interval` directly:
///
/// ```ignore
/// vec![with_interval(my_system, Duration::from_millis(250))]
/// ```
pub fn with_interval(system: Box<dyn System>, interval: Duration) -> Box<dyn System> {
    Box::new(WithInterval {
        inner: system,
        interval,
    })
}

/// Wrap a boxed system as low priority: runs once per second instead of
/// every update cycle. See [`LOW_PRIORITY_INTERVAL`].
pub fn low_priority(system: Box<dyn System>) -> Box<dyn System> {
    with_interval(system, LOW_PRIORITY_INTERVAL)
}

impl System for Box<dyn System> {
    fn name(&self) -> &str {
        (**self).name()
    }
    fn access(&self) -> &ComponentAccess {
        (**self).access()
    }
    fn run(&mut self, world: &crate::World, commands: &mut Commands) {
        (**self).run(world, commands)
    }
    fn desired_interval(&self) -> Option<Duration> {
        (**self).desired_interval()
    }
}
