use crate::wipi::java::{JavaClassProto, JavaMethodProto, Jvm};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut dyn Jvm) {
        log::debug!("Jlet::<init>");
    }
}
