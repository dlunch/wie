use alloc::sync::Arc;

use wie_backend::System;

use jvm::Jvm;

pub struct KtfContext {
    jvm: Option<Arc<Jvm>>,
}

impl KtfContext {
    pub fn new() -> Self {
        Self { jvm: None }
    }
}

pub trait KtfContextExt {
    fn jvm(&mut self) -> Arc<Jvm>;
    fn set_jvm(&mut self, jvm: Jvm);
}

impl KtfContextExt for System {
    fn jvm(&mut self) -> Arc<Jvm> {
        let context = self.context();
        let context = (*context).downcast_ref::<KtfContext>().unwrap();

        context.jvm.as_ref().unwrap().clone()
    }

    fn set_jvm(&mut self, jvm: Jvm) {
        let mut context = self.context();
        let context = (*context).downcast_mut::<KtfContext>().unwrap();

        context.jvm = Some(Arc::new(jvm))
    }
}
