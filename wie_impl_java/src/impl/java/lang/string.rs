use alloc::{
    str,
    string::{String as RustString, ToString},
    vec,
    vec::Vec,
};

use bytemuck::{cast_slice, cast_vec};

use jvm::JavaChar;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodFlag, JavaMethodProto},
    handle::{Array, JvmClassInstanceHandle},
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
        this: JvmClassInstanceHandle<Self>,
        value: JvmClassInstanceHandle<Array<i8>>,
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
        this: JvmClassInstanceHandle<Self>,
        value: JvmClassInstanceHandle<Array<u16>>,
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
        mut this: JvmClassInstanceHandle<Self>,
        value: JvmClassInstanceHandle<Array<u16>>,
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:?}, {:?}, {}, {})", &this, &value, offset, count);

        let mut array = context.jvm().instantiate_array("C", count as _).await?;
        context.jvm().put_field(&mut this, "value", "[C", array.clone())?;

        let data: Vec<JavaChar> = context.jvm().load_array(&value, offset as _, count as _)?;
        context.jvm().store_array(&mut array, 0, data)?; // TODO we should store value, offset, count like in java

        Ok(())
    }

    async fn init_with_partial_byte_array(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        value: JvmClassInstanceHandle<Array<i8>>,
        offset: i32,
        count: i32,
    ) -> JavaResult<()> {
        tracing::debug!("java.lang.String::<init>({:?}, {:?}, {}, {})", &this, &value, offset, count);

        let bytes: Vec<i8> = context.jvm().load_array(&value, offset as _, count as _)?;
        let string = context.system().decode_str(cast_slice(&bytes));

        let utf16 = string.encode_utf16().collect::<Vec<_>>();

        let mut array = context.jvm().instantiate_array("C", utf16.len()).await?;
        context.jvm().store_array(&mut array, 0, utf16)?;

        context
            .jvm()
            .invoke_special(&this, "java/lang/String", "<init>", "([C)V", [array.into()])
            .await?;

        Ok(())
    }

    async fn equals(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>, other: JvmClassInstanceHandle<Self>) -> JavaResult<bool> {
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

    async fn char_at(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>, index: i32) -> JavaResult<u16> {
        tracing::debug!("java.lang.String::charAt({:?}, {})", &this, index);

        let value = context.jvm().get_field(&this, "value", "[C")?;

        Ok(context.jvm().load_array(&value, index as _, 1)?[0])
    }

    async fn concat(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        other: JvmClassInstanceHandle<Self>,
    ) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::debug!("java.lang.String::concat({:?}, {:?})", &this, &other);

        let this_string = Self::to_rust_string(context, &this)?;
        let other_string = Self::to_rust_string(context, &other)?;

        let concat = this_string + &other_string;

        Self::from_rust_string(context, &concat).await
    }

    async fn get_bytes(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<JvmClassInstanceHandle<Array<i8>>> {
        tracing::debug!("java.lang.String::getBytes({:?})", &this);

        let string = Self::to_rust_string(context, &this)?;

        let bytes = context.system().encode_str(&string);
        let bytes: Vec<i8> = cast_vec(bytes);

        let mut byte_array = context.jvm().instantiate_array("B", bytes.len()).await?;
        context.jvm().store_array(&mut byte_array, 0, bytes)?;

        Ok(byte_array.into())
    }

    async fn length(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::length({:?})", &this);

        let value = context.jvm().get_field(&this, "value", "[C")?;

        Ok(context.jvm().array_length(&value)? as _)
    }

    async fn substring(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        begin_index: i32,
    ) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::debug!("java.lang.String::substring({:?}, {})", &this, begin_index);

        let string = Self::to_rust_string(context, &this)?;

        let substr = string.chars().skip(begin_index as usize).collect::<RustString>(); // TODO buffer sharing

        Self::from_rust_string(context, &substr).await
    }

    async fn substring_with_end(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        begin_index: i32,
        end_index: i32,
    ) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::debug!("java.lang.String::substring({:?}, {}, {})", &this, begin_index, end_index);

        let string = Self::to_rust_string(context, &this)?;

        let substr = string
            .chars()
            .skip(begin_index as usize)
            .take(end_index as usize - begin_index as usize)
            .collect::<RustString>(); // TODO buffer sharing

        Self::from_rust_string(context, &substr).await
    }

    async fn value_of_integer(context: &mut dyn JavaContext, value: i32) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::debug!("java.lang.String::valueOf({})", value);

        let string = value.to_string();

        Self::from_rust_string(context, &string).await
    }

    async fn value_of_object(context: &mut dyn JavaContext, value: JvmClassInstanceHandle<Object>) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::warn!("stub java.lang.String::valueOf({:?})", &value);

        // TODO Object.toString()

        Self::from_rust_string(context, "").await
    }

    async fn index_of_with_from(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        str: JvmClassInstanceHandle<Self>,
        from_index: i32,
    ) -> JavaResult<i32> {
        tracing::debug!("java.lang.String::indexOf({:?}, {:?})", &this, &str);

        let this_string = Self::to_rust_string(context, &this)?;
        let str_string = Self::to_rust_string(context, &str)?;

        let index = this_string[from_index as usize..].find(&str_string).map(|x| x as i32 + from_index);

        Ok(index.unwrap_or(-1))
    }

    async fn trim(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::debug!("java.lang.String::trim({:?})", &this);

        let string = Self::to_rust_string(context, &this)?;

        let trimmed = string.trim().to_string();

        Self::from_rust_string(context, &trimmed).await // TODO buffer sharing
    }

    pub fn to_rust_string(context: &mut dyn JavaContext, instance: &JvmClassInstanceHandle<String>) -> JavaResult<RustString> {
        let value = context.jvm().get_field(instance, "value", "[C")?;

        let length = context.jvm().array_length(&value)?;
        let string: Vec<JavaChar> = context.jvm().load_array(&value, 0, length)?;

        Ok(RustString::from_utf16(&string)?)
    }

    pub async fn from_rust_string(context: &mut dyn JavaContext, string: &str) -> JavaResult<JvmClassInstanceHandle<Self>> {
        let utf16 = string.encode_utf16().collect::<Vec<_>>();

        Self::from_utf16(context, utf16).await
    }

    pub async fn from_utf16(context: &mut dyn JavaContext, data: Vec<u16>) -> JavaResult<JvmClassInstanceHandle<Self>> {
        let mut java_value = context.jvm().instantiate_array("C", data.len()).await?;

        context.jvm().store_array(&mut java_value, 0, data.to_vec())?;

        let instance = context.jvm().new_class("java/lang/String", "([C)V", (java_value,)).await?;

        Ok(instance.into())
    }
}
