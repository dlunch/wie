use alloc::vec;

use crate::{
    array::Array,
    base::{JavaClassProto, JavaFieldProto, JavaMethodFlag, JavaMethodProto},
    r#impl::java::lang::String,
    JavaContext, JavaObjectProxy, JavaResult,
};

// class java.lang.StringBuffer
pub struct StringBuffer {}

impl StringBuffer {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "append",
                    "(Ljava/lang/String;)Ljava/lang/StringBuffer;",
                    Self::append_string,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new("toString", "()Ljava/lang/String;", Self::to_string, JavaMethodFlag::NONE),
            ],
            fields: vec![
                JavaFieldProto::new("value", "[C", crate::JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("count", "I", crate::JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<StringBuffer>) -> JavaResult<()> {
        tracing::debug!("java.lang.StringBuffer::<init>({:#x})", this.ptr_instance);

        let java_value_array = context.instantiate_array("C", 16).await?;
        context.put_field(&this.cast(), "value", java_value_array.ptr_instance)?;
        context.put_field(&this.cast(), "count", 0)?;

        Ok(())
    }

    async fn append_string(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<StringBuffer>,
        string: JavaObjectProxy<String>,
    ) -> JavaResult<JavaObjectProxy<StringBuffer>> {
        tracing::debug!("java.lang.StringBuffer::append({:#x}, {:#x})", this.ptr_instance, string.ptr_instance);
        let current_count = context.get_field(&this.cast(), "count")?;

        let java_value_to_add_array = JavaObjectProxy::new(context.get_field(&string.cast(), "value")?);
        let count_to_add = context.call_method(&string.cast(), "length", "()I", &[]).await?;
        let value_to_add = context.load_array_u8(&java_value_to_add_array, 0, count_to_add)?; // should be u16

        StringBuffer::ensure_capacity(context, &this, current_count + count_to_add).await?;

        let java_value_aray = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?);
        context.store_array_u8(&java_value_aray, current_count, &value_to_add)?;
        context.put_field(&this.cast(), "count", current_count + count_to_add)?;

        Ok(this)
    }

    async fn to_string(context: &mut dyn JavaContext, this: JavaObjectProxy<StringBuffer>) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!("java.lang.StringBuffer::toString({:#x})", this.ptr_instance);

        let java_value = JavaObjectProxy::<Array>::new(context.get_field(&this.cast(), "value")?);
        let count = context.get_field(&this.cast(), "count")?;

        let string = context.instantiate("Ljava/lang/String;").await?.cast();
        context
            .call_method(&string.cast(), "<init>", "([CII)V", &[java_value.ptr_instance, 0, count])
            .await?;

        Ok(string)
    }

    async fn ensure_capacity(context: &mut dyn JavaContext, this: &JavaObjectProxy<StringBuffer>, capacity: u32) -> JavaResult<()> {
        let java_value_array = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?);
        let current_capacity = context.array_length(&java_value_array)?;

        if current_capacity < capacity {
            let old_values = context.load_array_u8(&java_value_array, 0, current_capacity)?;
            let new_capacity = capacity * 2;

            let java_new_value_array = context.instantiate_array("C", new_capacity).await?;
            context.put_field(&this.cast(), "value", java_new_value_array.ptr_instance)?;
            context.store_array_u8(&java_new_value_array, 0, &old_values)?;
            context.destroy(java_value_array.cast())?;
        }

        Ok(())
    }
}
