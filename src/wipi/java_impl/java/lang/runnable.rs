use crate::wipi::java_impl::JavaClassImpl;

// interface java.lang.Runnable
pub struct Runnable {}

impl Runnable {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            name: "java/lang/Runnable".into(),
            methods: vec![],
        }
    }
}
