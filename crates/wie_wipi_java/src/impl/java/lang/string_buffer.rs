use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodAccessFlag, JavaMethodProto},
    JavaContext, JavaObjectProxy, JavaResult,
};

// class java.lang.StringBuffer
pub struct StringBuffer {}

impl StringBuffer {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new(
                    "append",
                    "(Ljava/lang/String;)Ljava/lang/StringBuffer;",
                    Self::append_string,
                    JavaMethodAccessFlag::NONE,
                ),
                JavaMethodProto::new("toString", "()Ljava/lang/String;", Self::to_string, JavaMethodAccessFlag::NONE),
            ],
            fields: vec![
                JavaFieldProto::new("value", "[C", crate::JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("count", "I", crate::JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::trace!("java.lang.StringBuffer::<init>({:#x})", instance.ptr_instance);

        let java_value_array = context.instantiate_array("C", 16)?;
        context.put_field(&instance, "value", java_value_array.ptr_instance)?;
        context.put_field(&instance, "count", 0)?;

        Ok(())
    }

    async fn append_string(context: &mut dyn JavaContext, instance: JavaObjectProxy, string: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::trace!(
            "stub java.lang.StringBuffer::append({:#x}, {:#x})",
            instance.ptr_instance,
            string.ptr_instance
        );
        let current_count = context.get_field(&instance, "count")?;

        let java_value_to_add_array = JavaObjectProxy::new(context.get_field(&string, "value")?);
        let count_to_add = context.call_method(&string, "length", "()I", &[]).await?;
        let value_to_add = context.load_array(&java_value_to_add_array, 0, count_to_add)?;

        StringBuffer::ensure_capacity(context, &instance, current_count + count_to_add)?;

        let java_value_aray = JavaObjectProxy::new(context.get_field(&instance, "value")?);
        context.store_array(&java_value_aray, current_count, &value_to_add)?;

        Ok(instance)
    }

    async fn to_string(context: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::trace!("java.lang.StringBuffer::toString({:#x})", instance.ptr_instance);

        let java_value = JavaObjectProxy::new(context.get_field(&instance, "value")?);
        let count = context.get_field(&instance, "count")?;

        let string = context.instantiate("Ljava/lang/String;")?;
        context
            .call_method(&string, "<init>", "([CII)V", &[java_value.ptr_instance, 0, count])
            .await?;

        Ok(string)
    }

    fn ensure_capacity(context: &mut dyn JavaContext, instance: &JavaObjectProxy, capacity: u32) -> JavaResult<()> {
        let java_value_array = JavaObjectProxy::new(context.get_field(instance, "value")?);
        let current_capacity = context.array_length(&java_value_array)?;

        if current_capacity < capacity {
            let old_values = context.load_array(&java_value_array, 0, current_capacity)?;
            let new_capacity = capacity * 2;

            let java_new_value_array = context.instantiate_array("C", new_capacity)?;
            // TODO free existing array
            context.put_field(instance, "value", java_new_value_array.ptr_instance)?;
            context.store_array(&java_new_value_array, 0, &old_values)?;
        }

        Ok(())
    }
}
