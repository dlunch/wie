use core::cell::{Ref, RefCell};

use alloc::{boxed::Box, collections::BTreeMap, rc::Rc, vec::Vec};

use wie_base::{
    task::{Task, TaskStatus},
    Core,
};

use crate::Backend;

pub struct Scheduler {
    tasks: Vec<Rc<RefCell<Box<dyn Task>>>>,
    current_task: Option<Rc<RefCell<Box<dyn Task>>>>,
    sleeping_tasks: BTreeMap<u64, Rc<RefCell<Box<dyn Task>>>>,
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
        self.tasks.push(Rc::new(RefCell::new(Box::new(task))))
    }

    pub fn current_task(&self) -> Option<Ref<'_, Box<dyn Task>>> {
        self.current_task.as_ref().map(|x| x.borrow())
    }

    pub(crate) fn run(backend: Backend, core: &mut dyn Core) -> anyhow::Result<()> {
        loop {
            let tasks = backend.scheduler().tasks.drain(..).collect::<Vec<_>>();
            if tasks.is_empty() {
                break;
            }

            let mut new_tasks = Vec::with_capacity(tasks.len());

            for task in tasks {
                backend.scheduler().current_task = Some(task.clone());
                task.borrow().run(core)?;

                let task = backend.scheduler().current_task.take().unwrap();

                let status = task.borrow().status();

                if status == TaskStatus::Running {
                    new_tasks.push(task);
                } else if let TaskStatus::Sleeping(until) = status {
                    backend.scheduler().sleeping_tasks.insert(until, task);
                }
            }

            backend.scheduler().tasks.append(&mut new_tasks);
        }

        Ok(())
    }
}
