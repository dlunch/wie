use alloc::rc::Rc;

use wie_backend::System;

use jvm::Jvm;

pub struct KtfContext {
    jvm: Option<Rc<Jvm>>,
}

impl KtfContext {
    pub fn new() -> Self {
        Self { jvm: None }
    }
}

pub trait KtfContextExt {
    fn jvm(&mut self) -> Rc<Jvm>;
    fn set_jvm(&mut self, jvm: Jvm);
}

impl KtfContextExt for System {
    fn jvm(&mut self) -> Rc<Jvm> {
        let context = self.context();
        let context = (*context).downcast_ref::<KtfContext>().unwrap();

        context.jvm.as_ref().unwrap().clone()
    }

    fn set_jvm(&mut self, jvm: Jvm) {
        let mut context = self.context();
        let context = (*context).downcast_mut::<KtfContext>().unwrap();

        context.jvm = Some(Rc::new(jvm))
    }
}
