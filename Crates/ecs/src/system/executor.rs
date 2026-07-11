use crate::World;
use crate::system::commands::Commands;
use crate::system::system::System;

pub trait Executor: Send {
    fn execute(&self, batch: &mut [&mut dyn System], world: &World, commands: &mut Commands);
}

pub struct SnapshotExecutor;

impl Executor for SnapshotExecutor {
    fn execute(&self, batch: &mut [&mut dyn System], world: &World, commands: &mut Commands) {
        if batch.is_empty() {
            return;
        }
        if batch.len() == 1 {
            batch[0].run(world, commands);
            return;
        }
        // For parallel execution, we need to split commands or use a shared buffer.
        // For now, run sequentially to avoid command buffer contention.
        // TODO: parallel execution with per-system command buffers
        for system in batch.iter_mut() {
            system.run(world, commands);
        }
    }
}

impl Default for SnapshotExecutor {
    fn default() -> Self {
        Self
    }
}
