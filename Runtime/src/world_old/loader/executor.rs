use std::{error::Error, fmt::Debug};

use super::{Event, Loader};

pub struct LoaderExecutor {
    /// Total amount of tasks that can run in parallel.
    /// Defaults to the logical CPU core count divided by 4.
    simultaneous_executions: usize,
    /// List of all currently active loader tasks.
    /// I.e. tasks that are running.
    active_loaders: Option<Vec<Box<dyn Loader>>>,
    /// List of all scheduled/queued loader tasks.
    /// I.e. tasks waiting to be executed.
    scheduled_loaders: Option<Vec<Box<dyn Loader>>>,
}

impl LoaderExecutor {
    pub fn new(simultaneous_executions: Option<usize>) -> Self {
        Self {
            simultaneous_executions: simultaneous_executions.unwrap_or(num_cpus::get() / 4),
            active_loaders: Some(Vec::new()),
            scheduled_loaders: Some(Vec::new()),
        }
    }

    pub fn schedule_loader<L: Loader + Sized + 'static>(&mut self, loader: L) {
        let boxed = Box::new(loader);
        self.schedule_loader_boxed(boxed);
    }

    pub fn schedule_loader_boxed(&mut self, loader: Box<dyn Loader>) {
        let mut scheduled_loaders = self.scheduled_loaders.take().unwrap();
        scheduled_loaders.push(loader);
        self.scheduled_loaders = Some(scheduled_loaders);
    }

    pub fn cycle(&mut self) -> Vec<Result<Vec<Event>, Box<dyn Error + Send + Sync>>> {
        let finished_results = self.do_check_finished_loaders();
        self.do_start_new_loaders();
        finished_results
    }

    pub fn do_check_finished_loaders(
        &mut self,
    ) -> Vec<Result<Vec<Event>, Box<dyn Error + Send + Sync>>> {
        // Take the current list of active loaders
        let active_loaders = self.active_loaders.take().unwrap();

        // Partition based on if the loader is done or not
        let (done, remaining): (Vec<_>, Vec<_>) = active_loaders
            .into_iter()
            .partition(|x| x.is_done_processing());

        // Repopulate the active loaders
        self.active_loaders = Some(remaining);

        done.into_iter()
            .map(|mut x| x.finish_processing())
            .collect::<Vec<_>>()
    }

    pub fn do_start_new_loaders(&mut self) {
        if self.current_scheduled_loader_count() == 0
            || self.current_active_loader_count() >= self.simultaneous_executions()
        {
            return; // Nothing to do!
        }

        let empty_slots = self.simultaneous_executions() - self.current_active_loader_count();

        let current_scheduled_amount = self.current_scheduled_loader_count();
        let smaller_amount = if current_scheduled_amount > empty_slots {
            empty_slots
        } else {
            current_scheduled_amount
        };

        let mut scheduled_loaders = self.scheduled_loaders.take().unwrap();
        let mut workers_to_be_activated = scheduled_loaders
            .drain(0..smaller_amount)
            .collect::<Vec<_>>();
        self.scheduled_loaders = Some(scheduled_loaders);

        // Activate all selected
        workers_to_be_activated
            .iter_mut()
            .for_each(|x| x.begin_processing());

        // Enqueue all loaders
        let mut active_loaders = self.active_loaders.take().unwrap();
        active_loaders.extend(workers_to_be_activated);
        self.active_loaders = Some(active_loaders);
    }

    pub fn simultaneous_executions(&self) -> usize {
        self.simultaneous_executions
    }

    /// Sets the simultaneous executions count.
    /// I.e. the total amount of loaders being executed at the
    /// same time.
    ///
    /// # Panic
    /// Panics, if the supplied number is zero.  
    /// Any supplied number must be >= 1.
    pub fn set_simultaneous_executions(&mut self, simultaneous_executions: usize) {
        if simultaneous_executions == 0 {
            panic!("Simultaneous executions cannot be zero!");
        }

        self.simultaneous_executions = simultaneous_executions;
    }

    pub fn current_active_loader_count(&self) -> usize {
        self.active_loaders.as_ref().unwrap().len()
    }

    pub fn current_scheduled_loader_count(&self) -> usize {
        self.scheduled_loaders.as_ref().unwrap().len()
    }
}

impl Debug for LoaderExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let active_loaders = match &self.active_loaders {
            Some(x) => format!("Some(Workers = {})", x.len()),
            None => "None".to_string(),
        };
        let scheduled_loaders = match &self.scheduled_loaders {
            Some(x) => format!("Some(Workers = {})", x.len()),
            None => "None".to_string(),
        };

        f.debug_struct("LoaderExecutor")
            .field("simultaneous_executions", &self.simultaneous_executions)
            .field("active_loaders", &active_loaders)
            .field("scheduled_loaders", &scheduled_loaders)
            .finish()
    }
}
