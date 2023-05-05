use crate::wipi::java::JavaClassProto;

// interface java.lang.Runnable
pub struct Runnable {}

impl Runnable {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto { methods: vec![] }
    }
}
