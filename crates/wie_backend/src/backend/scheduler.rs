use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};

use wie_base::Core;

use crate::Backend;

#[derive(Eq, PartialEq)]
pub enum TaskStatus {
    Running,
    Sleeping(u64),
    Finished,
}

pub trait Task {
    fn run_some(&self, core: &mut dyn Core) -> anyhow::Result<()>;
    fn status(&self) -> TaskStatus;
    fn sleep(&self, core: &mut dyn Core, time: u64);
    fn r#yield(&self, core: &mut dyn Core);
}

pub struct Scheduler {
    tasks: Vec<Box<dyn Task>>,
    current_task: Option<Box<dyn Task>>,
    sleeping_tasks: BTreeMap<u64, Box<dyn Task>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_task: None,
            sleeping_tasks: BTreeMap::new(),
        }
    }

    pub fn schedule<T>(&mut self, task: T)
    where
        T: Task + 'static,
    {
        self.tasks.push(Box::new(task))
    }

    pub fn current_task(&self) -> &Option<Box<dyn Task>> {
        &self.current_task
    }

    pub(crate) fn run(backend: Backend, core: &mut dyn Core) -> anyhow::Result<()> {
        let mut scheduler = backend.scheduler();

        loop {
            let tasks = scheduler.tasks.drain(..).collect::<Vec<_>>();
            if tasks.is_empty() {
                break;
            }

            let mut new_tasks = Vec::with_capacity(tasks.len());

            for task in tasks {
                scheduler.current_task = Some(task);
                scheduler.current_task.as_ref().unwrap().run_some(core)?;

                let task = scheduler.current_task.take().unwrap();

                let status = task.status();

                if status == TaskStatus::Running {
                    new_tasks.push(task);
                } else if let TaskStatus::Sleeping(until) = status {
                    scheduler.sleeping_tasks.insert(until, task);
                }
            }

            scheduler.tasks.append(&mut new_tasks);
        }

        Ok(())
    }
}
