use alloc::{boxed::Box, vec::Vec};

use wie_base::Core;

use crate::Backend;

pub trait Task {
    fn run_some(&mut self, core: &mut dyn Core) -> anyhow::Result<()>;
    fn is_finished(&self) -> bool;
}

pub struct Scheduler {
    tasks: Vec<Box<dyn Task>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn schedule<T>(&mut self, task: T)
    where
        T: Task + 'static,
    {
        self.tasks.push(Box::new(task))
    }

    pub(crate) fn run(backend: Backend, core: &mut dyn Core) -> anyhow::Result<()> {
        loop {
            let tasks = backend.scheduler().tasks.drain(..).collect::<Vec<_>>();
            if tasks.is_empty() {
                break;
            }

            let mut new_tasks = Vec::with_capacity(tasks.len());

            for mut task in tasks {
                task.run_some(core)?;

                if !task.is_finished() {
                    new_tasks.push(task);
                }
            }

            backend.scheduler().tasks.append(&mut new_tasks);
        }

        Ok(())
    }
}
