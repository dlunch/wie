use core::cell::Ref;

use alloc::vec;

use bytemuck::cast_vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    handle::JvmClassInstanceHandle,
    r#impl::java::{io::InputStream, lang::String},
    JavaContext, JavaMethodFlag, JavaResult,
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

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.Class::<init>({:?})", &this);

        Ok(())
    }

    #[allow(clippy::await_holding_refcell_ref)] // We manually drop Ref https://github.com/rust-lang/rust-clippy/issues/6353
    async fn get_resource_as_stream(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        name: JvmClassInstanceHandle<String>,
    ) -> JavaResult<JvmClassInstanceHandle<InputStream>> {
        let name = String::to_rust_string(context, &name)?;
        tracing::debug!("java.lang.Class::getResourceAsStream({:?}, {})", &this, name);

        let normalized_name = if let Some(x) = name.strip_prefix('/') { x } else { &name };

        let id = context.system().resource().id(normalized_name);
        if let Some(id) = id {
            let system_clone = context.system().clone();
            let data = Ref::map(system_clone.resource(), |x| x.data(id));

            let array = context.jvm().instantiate_array("B", data.len() as _).await?;

            context.jvm().store_byte_array(&array, 0, cast_vec(data.to_vec()))?;

            let result = context.jvm().new_class("java/io/ByteArrayInputStream", "([B)V", (array,)).await?;

            Ok(result.into())
        } else {
            Ok(None.into())
        }
    }
}
