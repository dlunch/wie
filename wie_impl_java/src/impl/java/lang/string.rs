use alloc::{
    str,
    string::{String as RustString, ToString},
    vec,
    vec::Vec,
};

use bytemuck::{cast_slice, cast_vec};

use jvm::JavaChar;

use wie_backend::{decode_str, encode_str};

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodFlag, JavaMethodProto},
    proxy::{Array, JvmClassInstanceProxy},
    r#impl::java::lang::Object,
    JavaContext, JavaFieldAccessFlag, JavaResult,
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
        value: JvmClassInstanceProxy<Array<i8>>,
    ) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:?}, {:?})", &this, &value);

        let count = context.jvm().array_length(&value)? as i32;

        context
            .jvm()
            .invoke_special(&this, "java/lang/String", "<init>", "([BII)V", (value, 0, count))
            .await?;

        Ok(())
    }

    async fn init_with_char_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: JvmClassInstanceProxy<Array<u16>>,
    ) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:?}, {:?})", &this, &value);

        let count = context.jvm().array_length(&value)? as i32;

        context
            .jvm()
            .invoke_special(&this, "java/lang/String", "<init>", "([CII)V", (value, 0, count))
            .await?;

        Ok(())
    }

    async fn init_with_partial_char_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: JvmClassInstanceProxy<Array<u16>>,
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:?}, {:?}, {}, {})", &this, &value, offset, count);

        let array = context.jvm().instantiate_array("C", count as _).await?;
        context.jvm().put_field(&this, "value", "[C", array.clone())?;

        let data: Vec<JavaChar> = context.jvm().load_array(&value, offset as _, count as _)?;
        context.jvm().store_array(&array, 0, data)?; // TODO we should store value, offset, count like in java

        Ok(())
    }

    async fn init_with_partial_byte_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        value: JvmClassInstanceProxy<Array<i8>>,
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:?}, {:?}, {}, {})", &this, &value, offset, count);

        let bytes: Vec<i8> = context.jvm().load_array(&value, offset as _, count as _)?;
        let string = decode_str(cast_slice(&bytes));

        let utf16 = string.encode_utf16().collect::<Vec<_>>();

        let array = context.jvm().instantiate_array("C", utf16.len()).await?;
        context.jvm().store_array(&array, 0, utf16)?;

        context
            .jvm()
            .invoke_special(&this, "java/lang/String", "<init>", "([C)V", [array.into()])
            .await?;

        Ok(())
    }

    async fn equals(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, other: JvmClassInstanceProxy<Self>) -> JavaResult<bool> {
        tracing::debug!("java.lang.String::equals({:?}, {:?})", &this, &other);

        // TODO Object.equals()

        let other_string = Self::to_rust_string(context, &other)?;
        let this_string = Self::to_rust_string(context, &this)?;

        if this_string == other_string {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn char_at(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, index: i32) -> JavaResult<u16> {
        tracing::debug!("java.lang.String::charAt({:?}, {})", &this, index);

        let value = context.jvm().get_field(&this, "value", "[C")?;

        Ok(context.jvm().load_array(&value, index as _, 1)?[0])
    }

    async fn concat(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        other: JvmClassInstanceProxy<Self>,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.String::concat({:?}, {:?})", &this, &other);

        let this_string = Self::to_rust_string(context, &this)?;
        let other_string = Self::to_rust_string(context, &other)?;

        let concat = this_string + &other_string;

        Self::from_rust_string(context, &concat).await
    }

    async fn get_bytes(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JvmClassInstanceProxy<Array<i8>>> {
        tracing::debug!("java.lang.String::getBytes({:?})", &this);

        let string = Self::to_rust_string(context, &this)?;

        let bytes = encode_str(&string);
        let bytes: Vec<i8> = cast_vec(bytes);

        let byte_array = context.jvm().instantiate_array("B", bytes.len()).await?;
        context.jvm().store_array(&byte_array, 0, bytes)?;

        Ok(byte_array.into())
    }

    async fn length(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::length({:?})", &this);

        let value = context.jvm().get_field(&this, "value", "[C")?;

        Ok(context.jvm().array_length(&value)? as _)
    }

    async fn substring(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        begin_index: i32,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.String::substring({:?}, {})", &this, begin_index);

        let string = Self::to_rust_string(context, &this)?;

        let substr = string.chars().skip(begin_index as usize).collect::<RustString>(); // TODO buffer sharing

        Self::from_rust_string(context, &substr).await
    }

    async fn substring_with_end(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        begin_index: i32,
        end_index: i32,
    ) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.String::substring({:?}, {}, {})", &this, begin_index, end_index);

        let string = Self::to_rust_string(context, &this)?;

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

    async fn value_of_object(context: &mut dyn JavaContext, value: JvmClassInstanceProxy<Object>) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::warn!("stub java.lang.String::valueOf({:?})", &value);

        // TODO Object.toString()

        Self::from_rust_string(context, "").await
    }

    async fn index_of_with_from(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        str: JvmClassInstanceProxy<Self>,
        from_index: i32,
    ) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::indexOf({:?}, {:?})", &this, &str);

        let this_string = Self::to_rust_string(context, &this)?;
        let str_string = Self::to_rust_string(context, &str)?;

        let index = this_string[from_index as usize..].find(&str_string).map(|x| x as i32 + from_index);

        Ok(index.unwrap_or(-1))
    }

    async fn trim(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::debug!("java.lang.String::trim({:?})", &this);

        let string = Self::to_rust_string(context, &this)?;

        let trimmed = string.trim().to_string();

        Self::from_rust_string(context, &trimmed).await // TODO buffer sharing
    }

    pub fn to_rust_string(context: &mut dyn JavaContext, instance: &JvmClassInstanceProxy<String>) -> JavaResult<RustString> {
        let value = context.jvm().get_field(instance, "value", "[C")?;

        let length = context.jvm().array_length(&value)?;
        let string: Vec<JavaChar> = context.jvm().load_array(&value, 0, length)?;

        Ok(RustString::from_utf16(&string)?)
    }

    pub async fn from_rust_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JvmClassInstanceProxy<Self>> {
        let utf16 = string.encode_utf16().collect::<Vec<_>>();

        Self::from_utf16(context, utf16).await
    }

    pub async fn from_utf16(context: &mut dyn JavaContext, data: Vec<u16>) -> JavaResult<JvmClassInstanceProxy<Self>> {
        let java_value = context.jvm().instantiate_array("C", data.len()).await?;

        context.jvm().store_array(&java_value, 0, data.to_vec())?;

        let instance = context.jvm().instantiate_class("java/lang/String").await?;

        context
            .jvm()
            .invoke_special(&instance, "java/lang/String", "<init>", "([C)V", [java_value.into()])
            .await?;

        Ok(instance.into())
    }
}
