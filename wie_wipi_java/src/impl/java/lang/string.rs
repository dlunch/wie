use alloc::{str, string::String as RustString, vec, vec::Vec};

use bytemuck::cast_slice;

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
                JavaMethodProto::new("substring", "(I)Ljava/lang/String;", Self::substring, JavaMethodFlag::NONE),
                JavaMethodProto::new("substring", "(II)Ljava/lang/String;", Self::substring_with_end, JavaMethodFlag::NONE),
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
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            value.ptr_instance,
            offset,
            count
        );

        let array = context.instantiate_array("C", count as _).await?;
        context.put_field(&this.cast(), "value", array.ptr_instance)?;

        let data = context.load_array_i8(&value.cast(), offset as _, count as _)?;
        context.store_array_i8(&array, 0, &data)?; // TODO we should store value, offset, count like in java

        Ok(())
    }

    async fn init_with_partial_byte_array(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<String>,
        value: JavaObjectProxy<Array>,
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            value.ptr_instance,
            offset,
            count
        );

        let array = context.instantiate_array("C", count as _).await?;
        context.put_field(&this.cast(), "value", array.ptr_instance)?;

        let data = context.load_array_i8(&value.cast(), offset as _, count as _)?; // TODO convert to char
        context.store_array_i8(&array, 0, &data)?; // TODO we should store value, offset, count like in java

        Ok(())
    }

    async fn char_at(context: &mut dyn JavaContext, this: JavaObjectProxy<String>, index: i32) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::charAt({:#x}, {})", this.ptr_instance, index);

        let value = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?);

        Ok(context.load_array_i8(&value, index as _, 1)?[0] as _) // should be u16
    }

    async fn get_bytes(context: &mut dyn JavaContext, this: JavaObjectProxy<String>) -> JavaResult<JavaObjectProxy<Array>> {
        tracing::debug!("java.lang.String::getBytes({:#x})", this.ptr_instance);

        let value = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?); // TODO convert to bytes..

        Ok(value)
    }

    async fn length(context: &mut dyn JavaContext, this: JavaObjectProxy<String>) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::length({:#x})", this.ptr_instance);

        let value = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?);

        Ok(context.array_length(&value)? as _)
    }

    async fn substring(context: &mut dyn JavaContext, this: JavaObjectProxy<String>, begin_index: i32) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!("java.lang.String::substring({:#x}, {})", this.ptr_instance, begin_index);

        let string = Self::to_rust_string(context, &this)?;

        let substr = &string[begin_index as usize..]; // TODO buffer sahring

        Self::to_java_string(context, substr).await
    }

    async fn substring_with_end(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<String>,
        begin_index: i32,
        end_index: i32,
    ) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!("java.lang.String::substring({:#x}, {}, {})", this.ptr_instance, begin_index, end_index);

        let string = Self::to_rust_string(context, &this)?;

        let substr = &string[begin_index as usize..end_index as usize]; // TODO buffer sahring

        Self::to_java_string(context, substr).await
    }

    pub fn to_rust_string(context: &mut dyn JavaContext, instance: &JavaObjectProxy<String>) -> JavaResult<RustString> {
        let java_value = JavaObjectProxy::new(context.get_field(&instance.cast(), "value")?);
        let length = context.array_length(&java_value)?;
        let string = context.load_array_i8(&java_value, 0, length)?;

        Ok(str::from_utf8(cast_slice(&string))?.into())
    }

    pub async fn to_java_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JavaObjectProxy<String>> {
        let bytes = string.bytes().collect::<Vec<_>>();
        let java_value = context.instantiate_array("C", bytes.len()).await?;
        context.store_array_i8(&java_value, 0, cast_slice(&bytes))?;

        let instance = context.instantiate("Ljava/lang/String;").await?.cast();
        context
            .call_method(&instance.cast(), "<init>", "([C)V", &[java_value.ptr_instance])
            .await?;

        Ok(instance)
    }
}
