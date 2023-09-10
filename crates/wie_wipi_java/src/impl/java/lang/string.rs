use alloc::{str, string::String as RustString, vec, vec::Vec};

use crate::{
    array::Array,
    base::{JavaClassProto, JavaFieldProto, JavaMethodFlag, JavaMethodProto},
    JavaContext, JavaFieldAccessFlag, JavaObjectProxy, JavaResult,
};

// class java.lang.String
pub struct String {}

impl String {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "([C)V", Self::init_with_char_array, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "([CII)V", Self::init_with_partial_char_array, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "([BII)V", Self::init_with_partial_byte_array, JavaMethodFlag::NONE),
                JavaMethodProto::new("charAt", "(I)C", Self::char_at, JavaMethodFlag::NONE),
                JavaMethodProto::new("getBytes", "()[B", Self::get_bytes, JavaMethodFlag::NONE),
                JavaMethodProto::new("length", "()I", Self::length, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("value", "[C", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init_with_char_array(context: &mut dyn JavaContext, this: JavaObjectProxy<String>, value: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:#x}, {:#x})", this.ptr_instance, value.ptr_instance,);

        context.put_field(&this.cast(), "value", value.ptr_instance)?;

        Ok(())
    }

    async fn init_with_partial_char_array(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<String>,
        value: JavaObjectProxy<Array>,
        offset: u32,
        count: u32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            value.ptr_instance,
            offset,
            count
        );

        let array = context.instantiate_array("C", count)?;
        context.put_field(&this.cast(), "value", array.ptr_instance)?;

        let data = context.load_array_u8(&value.cast(), offset, count)?;
        context.store_array_u8(&array, 0, &data)?; // TODO we should store value, offset, count like in java

        Ok(())
    }

    async fn init_with_partial_byte_array(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<String>,
        value: JavaObjectProxy<Array>,
        offset: u32,
        count: u32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            value.ptr_instance,
            offset,
            count
        );

        let array = context.instantiate_array("C", count)?;
        context.put_field(&this.cast(), "value", array.ptr_instance)?;

        let data = context.load_array_u8(&value.cast(), offset, count)?; // TODO convert to char
        context.store_array_u8(&array, 0, &data)?; // TODO we should store value, offset, count like in java

        Ok(())
    }

    async fn char_at(context: &mut dyn JavaContext, this: JavaObjectProxy<String>, index: u32) -> JavaResult<u32> {
        tracing::debug!("java.lang.String::charAt({:#x}, {})", this.ptr_instance, index);

        let value = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?);

        Ok(context.load_array_u8(&value, index, 1)?[0] as u32) // should be u16
    }

    async fn get_bytes(context: &mut dyn JavaContext, this: JavaObjectProxy<String>) -> JavaResult<JavaObjectProxy<Array>> {
        tracing::debug!("java.lang.String::getBytes({:#x})", this.ptr_instance);

        let value = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?); // TODO convert to bytes..

        Ok(value)
    }

    async fn length(context: &mut dyn JavaContext, this: JavaObjectProxy<String>) -> JavaResult<u32> {
        tracing::debug!("java.lang.String::length({:#x})", this.ptr_instance);

        let value = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?);

        context.array_length(&value)
    }

    pub fn to_rust_string(context: &mut dyn JavaContext, instance: &JavaObjectProxy<String>) -> JavaResult<RustString> {
        let java_value = JavaObjectProxy::new(context.get_field(&instance.cast(), "value")?);
        let length = context.array_length(&java_value)?;
        let string = context.load_array_u8(&java_value, 0, length)?;

        Ok(str::from_utf8(&string)?.into())
    }

    pub async fn to_java_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JavaObjectProxy<String>> {
        let bytes = string.bytes().collect::<Vec<_>>();
        let java_value = context.instantiate_array("C", bytes.len() as u32)?;
        context.store_array_u8(&java_value, 0, &bytes)?;

        let instance = context.instantiate("Ljava/lang/String;")?.cast();
        context
            .call_method(&instance.cast(), "<init>", "([C)V", &[java_value.ptr_instance])
            .await?;

        Ok(instance)
    }
}
