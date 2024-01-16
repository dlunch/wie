use alloc::rc::Rc;

use wie_backend::SystemHandle;

use jvm::Jvm;

pub struct KtfContext {
    jvm: Option<Rc<Jvm>>,
}

impl KtfContext {
    pub fn new() -> Self {
        Self { jvm: None }
    }

    pub fn jvm(system: &mut SystemHandle) -> Rc<Jvm> {
        let context = system.context();
        let context = (*context).downcast_ref::<Self>().unwrap();

        context.jvm.as_ref().unwrap().clone()
    }

    pub fn set_jvm(system: &mut SystemHandle, jvm: Jvm) {
        let mut context = system.context();
        let context = (*context).downcast_mut::<Self>().unwrap();

        context.jvm = Some(Rc::new(jvm))
    }
}
