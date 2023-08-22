use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    r#impl::java::lang::{Object, String},
    string::from_java_string,
    JavaContext, JavaMethodAccessFlag, JavaObjectProxy, JavaResult,
};

// class java.lang.Class
pub struct Class {}

impl Class {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new(
                    "getResourceAsStream",
                    "(Ljava/lang/String;)Ljava/io/InputStream;",
                    Self::get_resource_as_stream,
                    JavaMethodAccessFlag::NONE,
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Class>) -> JavaResult<()> {
        log::warn!("stub java.lang.Class::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn get_resource_as_stream(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<Class>,
        name: JavaObjectProxy<String>,
    ) -> JavaResult<JavaObjectProxy<Object>> {
        log::warn!(
            "stub java.lang.Class::getResourceAsStream({:#x}, {:#x})",
            this.ptr_instance,
            name.ptr_instance
        );

        let name = from_java_string(context, &name)?;
        log::debug!("getResourceAsStream name: {}", name);

        let result = context.instantiate("Ljava/io/InputStream;")?.cast();
        context.call_method(&result.cast(), "<init>", "()V", &[]).await?;

        Ok(result)
    }
}
