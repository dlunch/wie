use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    Array,
};

// class org.kwis.msp.lcdui.Main
pub struct Main {}

impl Main {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("main", "([Ljava/lang/String;)V", Self::main, JavaMethodFlag::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Main>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn main(_: &mut dyn JavaContext, this: JavaObjectProxy<Main>, args: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::<init>({:#x}, {:#x})", this.ptr_instance, args.ptr_instance);

        Ok(())
    }
}
