use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::java::lang::Class,
};

// class java.lang.Object
pub struct Object {}

impl Object {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("getClass", "()Ljava/lang/Class;", Self::get_class, JavaMethodAccessFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Object>) -> JavaResult<()> {
        log::warn!("stub java.lang.Object::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn get_class(context: &mut dyn JavaContext, this: JavaObjectProxy<Object>) -> JavaResult<JavaObjectProxy<Class>> {
        log::warn!("stub java.lang.Object::get_class({:#x})", this.ptr_instance);

        let result = context.instantiate("Ljava/lang/Class;")?.cast();

        Ok(result)
    }
}
