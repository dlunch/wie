use alloc::vec;

use crate::{
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

    async fn init_with_char_array(context: &mut dyn JavaContext, instance: JavaObjectProxy, value: JavaObjectProxy) -> JavaResult<()> {
        log::trace!("java.lang.String::<init>({:#x}, {:#x})", instance.ptr_instance, value.ptr_instance,);

        context.put_field(&instance, "value", value.ptr_instance)?;

        Ok(())
    }

    async fn init_with_partial_char_array(
        context: &mut dyn JavaContext,
        instance: JavaObjectProxy,
        value: JavaObjectProxy,
        offset: u32,
        count: u32,
    ) -> JavaResult<()> {
        log::trace!(
            "java.lang.String::<init>({:#x}, {:#x}, {}, {})",
            instance.ptr_instance,
            value.ptr_instance,
            offset,
            count
        );

        let array = context.instantiate_array("C", count)?;
        context.put_field(&instance, "value", array.ptr_instance)?;

        let data = context.load_array(&value, offset, count)?;
        context.store_array(&array, 0, &data)?;

        Ok(())
    }

    async fn get_bytes(context: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::trace!("java.lang.String::getBytes({:#x})", instance.ptr_instance);

        let value = JavaObjectProxy::new(context.get_field(&instance, "value")?);

        Ok(value)
    }

    async fn length(context: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<u32> {
        log::trace!("java.lang.String::length({:#x})", instance.ptr_instance);

        let value = JavaObjectProxy::new(context.get_field(&instance, "value")?);

        Ok(context.array_length(&value)?)
    }
}
