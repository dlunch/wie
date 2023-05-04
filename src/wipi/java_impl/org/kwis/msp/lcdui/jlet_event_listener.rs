use crate::wipi::java_impl::JavaClassImpl;

// interface org.kwis.msp.lcdui.JletEventListener
pub struct JletEventListener {}

impl JletEventListener {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl { methods: vec![] }
    }
}
