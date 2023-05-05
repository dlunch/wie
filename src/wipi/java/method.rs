use super::Jvm;

pub trait JavaMethodImpl {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32;
}

pub struct JavaMethodHolder<F>(pub F);

trait JavaMethodCaller<T> {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32;
}

impl<F> JavaMethodImpl for JavaMethodHolder<F>
where
    F: Fn(&mut dyn Jvm, Vec<u32>) -> u32,
{
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32 {
        self.0(jvm, args)
    }
}
