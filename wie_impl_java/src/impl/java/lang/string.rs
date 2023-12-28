use alloc::{
    str,
    string::{String as RustString, ToString},
    vec,
    vec::Vec,
};

use jvm::{ClassInstanceRef, JavaValue};
use wie_backend::{decode_str, encode_str};

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodFlag, JavaMethodProto},
    proxy::{JvmArrayClassInstanceProxy, JvmClassInstanceProxy},
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

    async fn init_with_byte_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: JvmArrayClassInstanceProxy<i8>,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&value.class_instance)
        );

        let count = context.jvm().array_length(&value.class_instance)?;

        context
            .jvm()
            .invoke_method(
                &this.class_instance,
                "java/lang/String",
                "<init>",
                "([BII)V",
                &[
                    JavaValue::Object(Some(value.class_instance)),
                    JavaValue::Int(0),
                    JavaValue::Int(count as _),
                ],
            )
            .await?;

        Ok(())
    }

    async fn init_with_char_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: JvmArrayClassInstanceProxy<u16>,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&value.class_instance)
        );

        let count = context.jvm().array_length(&value.class_instance)?;

        context
            .jvm()
            .invoke_method(
                &this.class_instance,
                "java/lang/String",
                "<init>",
                "([CII)V",
                &[
                    JavaValue::Object(Some(value.class_instance)),
                    JavaValue::Int(0),
                    JavaValue::Int(count as _),
                ],
            )
            .await?;

        Ok(())
    }

    async fn init_with_partial_char_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: JvmArrayClassInstanceProxy<u16>,
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&value.class_instance),
            offset,
            count
        );

        let array = context.jvm().instantiate_array("C", count as _).await?;
        context
            .jvm()
            .put_field(&this.class_instance, "value", "[C", JavaValue::Object(Some(array.clone())))?;

        let data = context.jvm().load_array(&value.class_instance, offset as _, count as _)?;
        context.jvm().store_array(&array, 0, &data)?; // TODO we should store value, offset, count like in java

        Ok(())
    }

    async fn init_with_partial_byte_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: JvmArrayClassInstanceProxy<i8>,
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&value.class_instance),
            offset,
            count
        );

        let bytes = context.jvm().load_array(&value.class_instance, offset as _, count as _)?;
        let string = decode_str(&bytes.into_iter().map(|x| x.as_byte() as u8).collect::<Vec<_>>());

        let utf16 = string.encode_utf16().map(JavaValue::Char).collect::<Vec<_>>();

        let array = context.jvm().instantiate_array("C", utf16.len()).await?;
        context.jvm().store_array(&array, 0, &utf16)?;

        context
            .jvm()
            .invoke_method(
                &this.class_instance,
                "java/lang/String",
                "<init>",
                "([C)V",
                &[JavaValue::Object(Some(array))],
            )
            .await?;

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

        Ok(context.jvm().load_array(value.as_object_ref().unwrap(), index as _, 1)?[0].as_char() as _)
    }

    async fn concat(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        other: JvmClassInstanceProxy<Self>,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
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

    async fn get_bytes(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JvmArrayClassInstanceProxy<i8>> {
        tracing::debug!("java.lang.String::getBytes({:#x})", context.instance_raw(&this.class_instance));

        let string = Self::to_rust_string(context, &this.class_instance)?;

        let bytes = encode_str(&string);
        let bytes = bytes.into_iter().map(|x| JavaValue::Byte(x as _)).collect::<Vec<_>>();

        let byte_array = context.jvm().instantiate_array("B", bytes.len()).await?;
        context.jvm().store_array(&byte_array, 0, &bytes)?;

        Ok(JvmArrayClassInstanceProxy::new(byte_array))
    }

    async fn length(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::length({:#x})", context.instance_raw(&this.class_instance));

        let value = context.jvm().get_field(&this.class_instance, "value", "[C")?;

        Ok(context.jvm().array_length(value.as_object_ref().unwrap())? as _)
    }

    async fn substring(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        begin_index: i32,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
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
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
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

    async fn value_of_integer(context: &mut dyn JavaContext, value: i32) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.String::valueOf({})", value);

        let string = value.to_string();

        Self::from_rust_string(context, &string).await
    }

    async fn value_of_object(context: &mut dyn JavaContext, value: JavaObjectProxy<Object>) -> JavaResult<JvmClassInstanceProxy<Self>> {
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

    async fn trim(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.String::trim({:#x})", context.instance_raw(&this.class_instance));

        let string = Self::to_rust_string(context, &this.class_instance)?;

        let trimmed = string.trim().to_string();

        Self::from_rust_string(context, &trimmed).await // TODO buffer sharing
    }

    pub fn to_rust_string(context: &mut dyn JavaContext, instance: &ClassInstanceRef) -> JavaResult<RustString> {
        let value = context.jvm().get_field(instance, "value", "[C")?;

        let length = context.jvm().array_length(value.as_object_ref().unwrap())?;
        let string = context.jvm().load_array(value.as_object_ref().unwrap(), 0, length)?;

        Ok(RustString::from_utf16(&string.into_iter().map(|x| x.as_char()).collect::<Vec<_>>())?)
    }

    pub async fn from_rust_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JvmClassInstanceProxy<Self>> {
        let utf16 = string.encode_utf16().collect::<Vec<_>>();

        Self::from_utf16(context, &utf16).await
    }

    pub async fn from_utf16(context: &mut dyn JavaContext, data: &[u16]) -> JavaResult<JvmClassInstanceProxy<Self>> {
        let java_value = context.jvm().instantiate_array("C", data.len()).await?;

        let data = data.iter().map(|&x| JavaValue::Char(x)).collect::<Vec<_>>();
        context.jvm().store_array(&java_value, 0, &data)?;

        let instance = context.jvm().instantiate_class("java/lang/String").await?;

        context
            .jvm()
            .invoke_method(&instance, "java/lang/String", "<init>", "([C)V", &[JavaValue::Object(Some(java_value))])
            .await?;

        Ok(JvmClassInstanceProxy::new(instance))
    }
}
