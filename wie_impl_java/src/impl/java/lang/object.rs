use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::java::lang::Class,
};

// class java.lang.Object
pub struct Object {}

impl Object {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: None,
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClass", "()Ljava/lang/Class;", Self::get_class, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Object>) -> JavaResult<()> {
        tracing::debug!("java.lang.Object::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn get_class(context: &mut dyn JavaContext, this: JavaObjectProxy<Object>) -> JavaResult<JavaObjectProxy<Class>> {
        tracing::warn!("stub java.lang.Object::get_class({:#x})", this.ptr_instance);

        let result = context.jvm().instantiate_class("java/lang/Class").await?;
        let result = JavaObjectProxy::new(context.instance_raw(&result));

        context.call_method(&result, "<init>", "()V", &[]).await?;

        Ok(result.cast())
    }
}
