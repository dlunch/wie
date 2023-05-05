use super::Jvm;

pub trait JavaMethodBody {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32;
}

struct JavaMethodHolder<F>(pub F);

trait JavaMethodCaller<T> {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32;
}

impl<F> JavaMethodBody for JavaMethodHolder<F>
where
    F: Fn(&mut dyn Jvm, Vec<u32>) -> u32,
{
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32 {
        self.0(jvm, args)
    }
}

pub trait JavaMethodImpl<F> {
    fn into_body(self) -> Box<dyn JavaMethodBody>;
}

impl<F> JavaMethodImpl<F> for F
where
    F: Fn(&mut dyn Jvm, Vec<u32>) -> u32 + 'static,
{
    fn into_body(self) -> Box<dyn JavaMethodBody> {
        Box::new(JavaMethodHolder(self))
    }
}
