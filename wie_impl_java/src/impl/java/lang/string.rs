use alloc::{
    str,
    string::{String as RustString, ToString},
    vec,
    vec::Vec,
};

use bytemuck::cast_slice;

use jvm::{ClassInstanceRef, JavaValue};
use wie_backend::{decode_str, encode_str};

use crate::{
    array::Array,
    base::{JavaClassProto, JavaFieldProto, JavaMethodFlag, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
    r#impl::java::lang::Object,
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
                JavaMethodProto::new("<init>", "([B)V", Self::init_with_byte_array, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "([C)V", Self::init_with_char_array, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "([CII)V", Self::init_with_partial_char_array, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "([BII)V", Self::init_with_partial_byte_array, JavaMethodFlag::NONE),
                JavaMethodProto::new("equals", "(Ljava/lang/Object;)Z", Self::equals, JavaMethodFlag::NONE),
                JavaMethodProto::new("charAt", "(I)C", Self::char_at, JavaMethodFlag::NONE),
                JavaMethodProto::new("getBytes", "()[B", Self::get_bytes, JavaMethodFlag::NONE),
                JavaMethodProto::new("length", "()I", Self::length, JavaMethodFlag::NONE),
                JavaMethodProto::new("concat", "(Ljava/lang/String;)Ljava/lang/String;", Self::concat, JavaMethodFlag::NONE),
                JavaMethodProto::new("substring", "(I)Ljava/lang/String;", Self::substring, JavaMethodFlag::NONE),
                JavaMethodProto::new("substring", "(II)Ljava/lang/String;", Self::substring_with_end, JavaMethodFlag::NONE),
                JavaMethodProto::new("valueOf", "(I)Ljava/lang/String;", Self::value_of_integer, JavaMethodFlag::STATIC),
                JavaMethodProto::new(
                    "valueOf",
                    "(Ljava/lang/Object;)Ljava/lang/String;",
                    Self::value_of_object,
                    JavaMethodFlag::STATIC,
                ),
                JavaMethodProto::new("indexOf", "(Ljava/lang/String;I)I", Self::index_of_with_from, JavaMethodFlag::NONE),
                JavaMethodProto::new("trim", "()Ljava/lang/String;", Self::trim, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("value", "[C", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init_with_byte_array(context: &mut dyn JavaContext, this: JavaObjectProxy<String>, value: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:#x}, {:#x})", this.ptr_instance, value.ptr_instance,);

        let count = context.array_length(&value)?;

        context
            .call_method(&this.cast(), "<init>", "([BII)V", &[value.ptr_instance, 0, count as _])
            .await?;

        Ok(())
    }

    async fn init_with_char_array(context: &mut dyn JavaContext, this: JavaObjectProxy<String>, value: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:#x}, {:#x})", this.ptr_instance, value.ptr_instance,);

        let count = context.array_length(&value)?;

        context
            .call_method(&this.cast(), "<init>", "([CII)V", &[value.ptr_instance, 0, count as _])
            .await?;

        Ok(())
    }

    async fn init_with_partial_char_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<String>,
        value: JavaObjectProxy<Array>,
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            context.instance_raw(&this.class_instance),
            value.ptr_instance,
            offset,
            count
        );

        let array = context.instantiate_array("C", count as _).await?;
        let array = context.instance_from_raw(array.ptr_instance);
        context
            .jvm()
            .put_field(&this.class_instance, "value", "[C", JavaValue::Object(Some(array.clone())))?;

        let data = context.load_array_i16(&value.cast(), offset as _, count as _)?;
        context.store_array_i16(&JavaObjectProxy::new(context.instance_raw(&array)), 0, &data)?; // TODO we should store value, offset, count like in java

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

        let bytes = context.load_array_i8(&value.cast(), offset as _, count as _)?;
        let string = decode_str(cast_slice(&bytes));

        let utf16 = string.encode_utf16().collect::<Vec<_>>();

        let array = context.instantiate_array("C", utf16.len()).await?;
        context.store_array_i16(&array, 0, cast_slice(&utf16))?;

        context.call_method(&this.cast(), "<init>", "([C)V", &[array.ptr_instance]).await?;

        Ok(())
    }

    async fn equals(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, other: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!(
            "java.lang.String::equals({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&other.class_instance)
        );

        // TODO Object.equals()

        let other_string = Self::to_rust_string(context, &other.class_instance)?;
        let this_string = Self::to_rust_string(context, &this.class_instance)?;

        if this_string == other_string {
            Ok(1)
        } else {
            Ok(0) // TODO boolean type
        }
    }

    async fn char_at(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, index: i32) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::charAt({:#x}, {})", context.instance_raw(&this.class_instance), index);

        let value = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        let value = JavaObjectProxy::new(context.instance_raw(value.as_object().unwrap()));

        Ok(context.load_array_i16(&value, index as _, 1)?[0] as _)
    }

    async fn concat(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        other: JvmClassInstanceProxy<Self>,
    ) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!(
            "java.lang.String::concat({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&other.class_instance)
        );

        let this_string = Self::to_rust_string(context, &this.class_instance)?;
        let other_string = Self::to_rust_string(context, &other.class_instance)?;

        let concat = this_string + &other_string;

        Self::from_rust_string(context, &concat).await
    }

    async fn get_bytes(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JavaObjectProxy<Array>> {
        tracing::debug!("java.lang.String::getBytes({:#x})", context.instance_raw(&this.class_instance));

        let value = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        let value = JavaObjectProxy::new(context.instance_raw(value.as_object().unwrap()));

        let len = context.array_length(&value)?;
        let utf16 = context.load_array_i16(&value, 0, len)?;

        let bytes = encode_str(&RustString::from_utf16(cast_slice(&utf16))?);

        let byte_array = context.instantiate_array("B", bytes.len()).await?;
        context.store_array_i8(&byte_array, 0, cast_slice(&bytes))?;

        Ok(byte_array)
    }

    async fn length(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::length({:#x})", context.instance_raw(&this.class_instance));

        let value = context.jvm().get_field(&this.class_instance, "value", "[C")?;
        let value = JavaObjectProxy::new(context.instance_raw(value.as_object().unwrap()));

        Ok(context.array_length(&value)? as _)
    }

    async fn substring(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, begin_index: i32) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!(
            "java.lang.String::substring({:#x}, {})",
            context.instance_raw(&this.class_instance),
            begin_index
        );

        let string = Self::to_rust_string(context, &this.class_instance)?;

        let substr = string.chars().skip(begin_index as usize).collect::<RustString>(); // TODO buffer sharing

        Self::from_rust_string(context, &substr).await
    }

    async fn substring_with_end(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        begin_index: i32,
        end_index: i32,
    ) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!(
            "java.lang.String::substring({:#x}, {}, {})",
            context.instance_raw(&this.class_instance),
            begin_index,
            end_index
        );

        let string = Self::to_rust_string(context, &this.class_instance)?;

        let substr = string
            .chars()
            .skip(begin_index as usize)
            .take(end_index as usize - begin_index as usize)
            .collect::<RustString>(); // TODO buffer sharing

        Self::from_rust_string(context, &substr).await
    }

    async fn value_of_integer(context: &mut dyn JavaContext, value: i32) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!("java.lang.String::valueOf({})", value);

        let string = value.to_string();

        Self::from_rust_string(context, &string).await
    }

    async fn value_of_object(context: &mut dyn JavaContext, value: JavaObjectProxy<Object>) -> JavaResult<JavaObjectProxy<String>> {
        tracing::warn!("stub java.lang.String::valueOf({:#x})", value.ptr_instance);

        // TODO Object.toString()

        Self::from_rust_string(context, "").await
    }

    async fn index_of_with_from(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        str: JvmClassInstanceProxy<Self>,
        from_index: i32,
    ) -> JavaResult<i32> {
        tracing::debug!(
            "java.lang.String::indexOf({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&str.class_instance)
        );

        let this_string = Self::to_rust_string(context, &this.class_instance)?;
        let str_string = Self::to_rust_string(context, &str.class_instance)?;

        let index = this_string[from_index as usize..].find(&str_string).map(|x| x as i32 + from_index);

        Ok(index.unwrap_or(-1))
    }

    async fn trim(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JavaObjectProxy<String>> {
        tracing::debug!("java.lang.String::trim({:#x})", context.instance_raw(&this.class_instance));

        let string = Self::to_rust_string(context, &this.class_instance)?;

        let trimmed = string.trim().to_string();

        Self::from_rust_string(context, &trimmed).await // TODO buffer sharing
    }

    pub fn to_rust_string(context: &mut dyn JavaContext, instance: &ClassInstanceRef) -> JavaResult<RustString> {
        let value = context.jvm().get_field(instance, "value", "[C")?;
        let java_value = JavaObjectProxy::new(context.instance_raw(value.as_object().unwrap()));

        let length = context.array_length(&java_value)?;
        let string = context.load_array_i16(&java_value, 0, length)?;

        Ok(RustString::from_utf16(cast_slice(&string))?)
    }

    pub async fn from_rust_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JavaObjectProxy<String>> {
        let utf16 = string.encode_utf16().collect::<Vec<_>>();

        Self::from_utf16(context, &utf16).await
    }

    pub async fn from_utf16(context: &mut dyn JavaContext, data: &[u16]) -> JavaResult<JavaObjectProxy<String>> {
        let java_value = context.instantiate_array("C", data.len()).await?;
        context.store_array_i16(&java_value, 0, cast_slice(data))?;

        let instance = context.instantiate("Ljava/lang/String;").await?.cast();
        context
            .call_method(&instance.cast(), "<init>", "([C)V", &[java_value.ptr_instance])
            .await?;

        Ok(instance)
    }
}
