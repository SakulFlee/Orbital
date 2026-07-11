use crate::World;
use crate::system::access::ComponentAccess;
use crate::system::commands::Commands;
use crate::system::executor::{Executor, SnapshotExecutor};
use crate::system::system::{IntoSystem, System};

pub struct Schedule {
    systems: Vec<Box<dyn System>>,
    executor: Box<dyn Executor>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            executor: Box::new(SnapshotExecutor),
        }
    }

    pub fn with_executor(executor: Box<dyn Executor>) -> Self {
        Self {
            systems: Vec::new(),
            executor,
        }
    }

    pub fn add_system<M, S>(&mut self, system: S)
    where
        S: IntoSystem<M, System = Box<dyn System>>,
    {
        self.systems.push(system.into_system());
    }

    /// Add a pre-boxed system directly (useful for systems returned from Module::setup).
    pub fn add_system_boxed(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    /// Returns the number of systems in this schedule.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    pub fn run(&mut self, world: &mut World) {
        if self.systems.is_empty() {
            return;
        }

        let batches = self.build_batches();
        let mut commands = Commands::new();

        for indices in batches {
            // Collect unique mutable references to systems in this batch
            let batch_ptrs: Vec<*mut dyn System> = indices
                .iter()
                .map(|&i| self.systems[i].as_mut() as *mut dyn System)
                .collect();
            let mut refs: Vec<&mut dyn System> =
                batch_ptrs.iter().map(|&p| unsafe { &mut *p }).collect();
            self.executor.execute(&mut refs, world, &mut commands);
        }

        commands.flush(world).expect("Commands flush failed");
    }

    pub fn build_batches(&self) -> Vec<Vec<usize>> {
        let n = self.systems.len();
        if n <= 1 {
            return (0..n).map(|i| vec![i]).collect();
        }

        // Greedy: place each system in the first batch it doesn't conflict with
        let mut batches: Vec<Vec<usize>> = Vec::new();
        let mut batch_access: Vec<ComponentAccess> = Vec::new();

        for idx in 0..n {
            let access = self.systems[idx].access();
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
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}
