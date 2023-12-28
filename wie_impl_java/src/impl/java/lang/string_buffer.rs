use alloc::{
    string::{String as RustString, ToString},
    vec,
    vec::Vec,
};

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaWord},
    proxy::JvmClassInstanceProxy,
    r#impl::java::lang::String,
    JavaContext, JavaResult,
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
        tracing::debug!("java.lang.StringBuffer::<init>({:?})", &this);

        let array = context.jvm().instantiate_array("C", 16).await?;
        context
            .jvm()
            .put_field(&this.class_instance, "value", "[C", JavaValue::Object(Some(array)))?;
        context.jvm().put_field(&this.class_instance, "count", "I", JavaValue::Int(0))?;

        Ok(())
    }

    async fn init_with_string(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        string: JvmClassInstanceProxy<String>,
    ) -> JavaResult<()> {
        tracing::debug!("java.lang.StringBuffer::<init>({:?}, {:?})", &this, &string,);

        let value_array = context.jvm().get_field(&string.class_instance, "value", "[C")?;
        let length = context.jvm().array_length(value_array.as_object_ref().unwrap())?;

        context.jvm().put_field(&this.class_instance, "value", "[C", value_array)?;
        context.jvm().put_field(&this.class_instance, "count", "I", JavaValue::Int(length as _))?;

        Ok(())
    }

    async fn append_string(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        string: JvmClassInstanceProxy<String>,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.StringBuffer::append({:?}, {:?})", &this, &string,);

        let string = String::to_rust_string(context, &string.class_instance)?;

        Self::append(context, &this, &string).await?;

        Ok(this)
    }

    async fn append_integer(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, value: i32) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.StringBuffer::append({:?}, {:?})", &this, value);

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
        tracing::debug!("java.lang.StringBuffer::append({:?}, {:?}, {:?})", &this, value_low, value_high);

        let digits = ((value_high as i64) << 32 | (value_low as i64)).to_string();

        Self::append(context, &this, &digits).await?;

        Ok(this)
    }

    async fn append_character(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: i32,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.StringBuffer::append({:?}, {:?})", &this, value);

        let value = RustString::from_utf16(&[value as u16])?;

        Self::append(context, &this, &value).await?;

        Ok(this)
    }

    async fn to_string(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JvmClassInstanceProxy<String>> {
        tracing::debug!("java.lang.StringBuffer::toString({:?})", &this);

        let java_value = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        let count = context.jvm().get_field(&this.class_instance, "count", "I")?;

        let string = context.jvm().instantiate_class("java/lang/String").await?;
        context
            .jvm()
            .invoke_method(&string, "java/lang/String", "<init>", "([CII)V", &[java_value, JavaValue::Int(0), count])
            .await?;

        Ok(JvmClassInstanceProxy::new(string))
    }

    async fn ensure_capacity(context: &mut dyn JavaContext, this: &JvmClassInstanceProxy<Self>, capacity: JavaWord) -> JavaResult<()> {
        let java_value_array = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        let current_capacity = context.jvm().array_length(java_value_array.as_object_ref().unwrap())?;

        if current_capacity < capacity {
            let old_values = context.jvm().load_array(java_value_array.as_object_ref().unwrap(), 0, current_capacity)?;
            let new_capacity = capacity * 2;

            let java_new_value_array = context.jvm().instantiate_array("C", new_capacity).await?;
            context
                .jvm()
                .put_field(&this.class_instance, "value", "[C", JavaValue::Object(Some(java_new_value_array.clone())))?;
            context.jvm().store_array(&java_new_value_array, 0, &old_values)?;
        }

        Ok(())
    }

    async fn append(context: &mut dyn JavaContext, this: &JvmClassInstanceProxy<Self>, string: &str) -> JavaResult<()> {
        let current_count = context.jvm().get_field(&this.class_instance, "count", "I")?.as_int();

        let value_to_add = string.encode_utf16().map(JavaValue::Char).collect::<Vec<_>>();
        let count_to_add = value_to_add.len() as i32;

        StringBuffer::ensure_capacity(context, this, (current_count + count_to_add) as _).await?;

        let java_value_array = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        context
            .jvm()
            .store_array(java_value_array.as_object_ref().unwrap(), current_count as _, &value_to_add)?;
        context
            .jvm()
            .put_field(&this.class_instance, "count", "I", JavaValue::Int(current_count + count_to_add))?;

        Ok(())
    }
}
