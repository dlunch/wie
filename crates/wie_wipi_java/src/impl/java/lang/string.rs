use alloc::vec;

use crate::{
    array::Array,
    base::{JavaClassProto, JavaFieldProto, JavaMethodAccessFlag, JavaMethodProto},
    JavaContext, JavaFieldAccessFlag, JavaObjectProxy, JavaResult,
};

// class java.lang.String
pub struct String {}

impl String {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "([C)V", Self::init_with_char_array, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("<init>", "([CII)V", Self::init_with_partial_char_array, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("getBytes", "()[B", Self::get_bytes, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("length", "()I", Self::length, JavaMethodAccessFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("value", "[C", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init_with_char_array(context: &mut dyn JavaContext, this: JavaObjectProxy<String>, value: JavaObjectProxy<Array>) -> JavaResult<()> {
        log::trace!("java.lang.String::<init>({:#x}, {:#x})", this.ptr_instance, value.ptr_instance,);

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
        log::trace!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            value.ptr_instance,
            offset,
            count
        );

        let array = context.instantiate_array("C", count)?;
        context.put_field(&this.cast(), "value", array.ptr_instance)?;

        let data = context.load_array(&value.cast(), offset, count)?;
        context.store_array(&array, 0, &data)?; // TODO we should store value, offset, count like in java

        Ok(())
    }

    async fn get_bytes(context: &mut dyn JavaContext, this: JavaObjectProxy<String>) -> JavaResult<JavaObjectProxy<Array>> {
        log::trace!("java.lang.String::getBytes({:#x})", this.ptr_instance);

        let value = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?); // TODO convert to bytes..

        Ok(value)
    }

    async fn length(context: &mut dyn JavaContext, this: JavaObjectProxy<String>) -> JavaResult<u32> {
        log::trace!("java.lang.String::length({:#x})", this.ptr_instance);

        let value = JavaObjectProxy::new(context.get_field(&this.cast(), "value")?);

        context.array_length(&value)
    }
}
