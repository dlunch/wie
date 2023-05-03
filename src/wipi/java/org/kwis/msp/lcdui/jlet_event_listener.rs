use crate::wipi::java::JavaClassImpl;

// interface org.kwis.msp.lcdui.JletEventListener
pub struct JletEventListener {}

impl JletEventListener {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            name: "org/kwis/msp/lcdui/JletEventListener".to_owned(),
            methods: vec![],
        }
    }
}
