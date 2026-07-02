use crate::system::system::System;
use crate::World;

pub trait Executor: Send {
    fn execute(&self, batch: &mut [&mut dyn System], world: &World);
}

pub struct SnapshotExecutor;

impl Executor for SnapshotExecutor {
    fn execute(&self, batch: &mut [&mut dyn System], world: &World) {
        if batch.is_empty() {
            return;
        }
        if batch.len() == 1 {
            batch[0].run(world);
            return;
        }
        rayon::scope(|s| {
            for system in batch.iter_mut() {
                s.spawn(|_| system.run(world));
            }
        });
    }
}

impl Default for SnapshotExecutor {
    fn default() -> Self {
        Self
    }
}
