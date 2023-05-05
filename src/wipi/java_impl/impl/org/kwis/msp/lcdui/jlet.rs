use crate::wipi::java_impl::{JavaClassProto, JavaMethodProto, Jvm};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        log::debug!("Jlet::<init>");

        0
    }
}
