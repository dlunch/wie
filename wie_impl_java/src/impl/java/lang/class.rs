use core::cell::Ref;

use alloc::{vec, vec::Vec};

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
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
        tracing::warn!("stub java.lang.Class::<init>({:?})", this.ptr_instance);

        Ok(())
    }

    #[allow(clippy::await_holding_refcell_ref)] // We manually drop Ref https://github.com/rust-lang/rust-clippy/issues/6353
    async fn get_resource_as_stream(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        name: JvmClassInstanceProxy<String>,
    ) -> JavaResult<JvmClassInstanceProxy<InputStream>> {
        let name = String::to_rust_string(context, &name.class_instance)?;
        tracing::debug!("java.lang.Class::getResourceAsStream({:?}, {})", &this, name);

        let normalized_name = if let Some(x) = name.strip_prefix('/') { x } else { &name };

        let id = context.backend().resource().id(normalized_name);
        if let Some(id) = id {
            let backend1 = context.backend().clone();
            let data = Ref::map(backend1.resource(), |x| x.data(id));

            let array = context.jvm().instantiate_array("B", data.len() as _).await?;

            let data = data.iter().map(|&x| JavaValue::Byte(x as _)).collect::<Vec<_>>();
            context.jvm().store_array(&array, 0, &data)?;
            drop(data);

            let result = context.jvm().instantiate_class("java/io/ByteArrayInputStream").await?;
            context
                .jvm()
                .invoke_method(
                    &result,
                    "java/io/ByteArrayInputStream",
                    "<init>",
                    "([B)V",
                    &[JavaValue::Object(Some(array))],
                )
                .await?;

            Ok(JvmClassInstanceProxy::new(result))
        } else {
            anyhow::bail!("No such instance") // TODO return null
        }
    }
}
