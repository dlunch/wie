use alloc::{
    string::{String as RustString, ToString},
    vec,
    vec::Vec,
};

use bytemuck::cast_slice;

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaWord},
    proxy::JvmClassInstanceProxy,
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
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_string, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "append",
                    "(Ljava/lang/String;)Ljava/lang/StringBuffer;",
                    Self::append_string,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new("append", "(I)Ljava/lang/StringBuffer;", Self::append_integer, JavaMethodFlag::NONE),
                JavaMethodProto::new("append", "(J)Ljava/lang/StringBuffer;", Self::append_long, JavaMethodFlag::NONE),
                JavaMethodProto::new("append", "(C)Ljava/lang/StringBuffer;", Self::append_character, JavaMethodFlag::NONE),
                JavaMethodProto::new("toString", "()Ljava/lang/String;", Self::to_string, JavaMethodFlag::NONE),
            ],
            fields: vec![
                JavaFieldProto::new("value", "[C", crate::JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("count", "I", crate::JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::debug!("java.lang.StringBuffer::<init>({:#x})", context.instance_raw(&this.class_instance));

        let array = context.instantiate_array("C", 16).await?;
        let java_value_array = context.instance_from_raw(array.ptr_instance);
        context
            .jvm()
            .put_field(&this.class_instance, "value", "[C", JavaValue::Object(Some(java_value_array)))?;
        context.jvm().put_field(&this.class_instance, "count", "I", JavaValue::Integer(0))?;

        Ok(())
    }

    async fn init_with_string(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        string: JvmClassInstanceProxy<String>,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.StringBuffer::<init>({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&string.class_instance),
        );

        let value_array = context.jvm().get_field(&string.class_instance, "value", "[C")?;
        let length = context.array_length(&JavaObjectProxy::new(context.instance_raw(value_array.as_object().unwrap())))?;

        context.jvm().put_field(&this.class_instance, "value", "[C", value_array)?;
        context
            .jvm()
            .put_field(&this.class_instance, "count", "I", JavaValue::Integer(length as _))?;

        Ok(())
    }

    async fn append_string(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        string: JvmClassInstanceProxy<String>,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!(
            "java.lang.StringBuffer::append({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&string.class_instance),
        );

        let string = String::to_rust_string(context, &string.class_instance)?;

        Self::append(context, &this, &string).await?;

        Ok(this)
    }

    async fn append_integer(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, value: i32) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!(
            "java.lang.StringBuffer::append({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            value
        );

        let digits = value.to_string();

        Self::append(context, &this, &digits).await?;

        Ok(this)
    }

    async fn append_long(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value_low: i32,
        value_high: i32,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!(
            "java.lang.StringBuffer::append({:#x}, {:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            value_low,
            value_high
        );

        let digits = ((value_high as i64) << 32 | (value_low as i64)).to_string();

        Self::append(context, &this, &digits).await?;

        Ok(this)
    }

    async fn append_character(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: i32,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!(
            "java.lang.StringBuffer::append({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            value
        );

        let value = RustString::from_utf16(&[value as u16])?;

        Self::append(context, &this, &value).await?;

        Ok(this)
    }

    async fn to_string(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!("java.lang.StringBuffer::toString({:#x})", context.instance_raw(&this.class_instance));

        let java_value = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        let count = context.jvm().get_field(&this.class_instance, "count", "I")?;

        let string = context.instantiate("Ljava/lang/String;").await?.cast();
        context
            .call_method(
                &string.cast(),
                "<init>",
                "([CII)V",
                &[context.instance_raw(java_value.as_object().unwrap()), 0, count.as_integer() as _],
            )
            .await?;

        Ok(string)
    }

    async fn ensure_capacity(context: &mut dyn JavaContext, this: &JvmClassInstanceProxy<Self>, capacity: JavaWord) -> JavaResult<()> {
        let java_value_array = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        let current_capacity = context.array_length(&JavaObjectProxy::new(context.instance_raw(java_value_array.as_object().unwrap())))?;

        if current_capacity < capacity {
            let old_values = context.load_array_i16(
                &JavaObjectProxy::new(context.instance_raw(java_value_array.as_object().unwrap())),
                0,
                current_capacity,
            )?;
            let new_capacity = capacity * 2;

            let java_new_value_array = context.instantiate_array("C", new_capacity).await?;
            let new_value = context.instance_from_raw(java_new_value_array.ptr_instance);
            context
                .jvm()
                .put_field(&this.class_instance, "value", "[C", JavaValue::Object(Some(new_value)))?;
            context.store_array_i16(&java_new_value_array, 0, &old_values)?;
        }

        Ok(())
    }

    async fn append(context: &mut dyn JavaContext, this: &JvmClassInstanceProxy<Self>, string: &str) -> JavaResult<()> {
        let current_count = context.jvm().get_field(&this.class_instance, "count", "I")?.as_integer();

        let value_to_add = string.encode_utf16().collect::<Vec<_>>();
        let count_to_add = value_to_add.len() as i32;

        StringBuffer::ensure_capacity(context, this, (current_count + count_to_add) as _).await?;

        let java_value_array = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        context.store_array_i16(
            &JavaObjectProxy::new(context.instance_raw(java_value_array.as_object().unwrap())),
            current_count as _,
            cast_slice(&value_to_add),
        )?;
        context
            .jvm()
            .put_field(&this.class_instance, "count", "I", JavaValue::Integer(current_count + count_to_add))?;

        Ok(())
    }
}
