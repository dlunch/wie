use core::cell::Ref;

use alloc::vec;

use bytemuck::cast_slice;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    r#impl::java::{io::InputStream, lang::String},
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
};

// class java.lang.Class
pub struct Class {}

impl Class {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "getResourceAsStream",
                    "(Ljava/lang/String;)Ljava/io/InputStream;",
                    Self::get_resource_as_stream,
                    JavaMethodFlag::NONE,
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Class>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.Class::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    #[allow(clippy::await_holding_refcell_ref)] // We manually drop Ref https://github.com/rust-lang/rust-clippy/issues/6353
    async fn get_resource_as_stream(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<Class>,
        name: JavaObjectProxy<String>,
    ) -> JavaResult<JavaObjectProxy<InputStream>> {
        let name = String::to_rust_string(context, &name)?;
        tracing::debug!("java.lang.Class::getResourceAsStream({:#x}, {})", this.ptr_instance, name);

        let normalized_name = if let Some(x) = name.strip_prefix('/') { x } else { &name };

        let id = context.backend().resource().id(normalized_name);
        if let Some(id) = id {
            let backend1 = context.backend().clone();
            let data = Ref::map(backend1.resource(), |x| x.data(id));

            let array = context.instantiate_array("B", data.len() as _).await?;
            context.store_array_i8(&array, 0, cast_slice(&data))?;
            drop(data);

            let result = context.instantiate("Ljava/io/ByteArrayInputStream;").await?.cast();
            context.call_method(&result.cast(), "<init>", "([B)V", &[array.ptr_instance]).await?;

            Ok(result)
        } else {
            Ok(JavaObjectProxy::new(0))
        }
    }
}
