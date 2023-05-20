use crate::Core;

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
