use alloc::{str, vec, vec::Vec};

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    JavaAccessFlag, JavaContext, JavaObjectProxy, JavaResult,
};

// class java.lang.String
pub struct String {}

impl String {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "(I)V", Self::init)],
            fields: vec![
                JavaFieldProto::new("value", "[C", JavaAccessFlag::NONE),
                JavaFieldProto::new("length", "I", JavaAccessFlag::NONE),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, instance: JavaObjectProxy, length: u32) -> JavaResult<()> {
        log::trace!("java.lang.String::<init>({:#x}, {})", instance.ptr_instance, length);

        let array = context.instantiate_array("I", length)?;
        context.put_field(&instance, "value", array.ptr_instance)?;
        context.put_field(&instance, "length", length)?;

        Ok(())
    }

    pub fn rust_str(context: &mut dyn JavaContext, instance: &JavaObjectProxy) -> JavaResult<alloc::string::String> {
        let name_array = JavaObjectProxy::new(context.get_field(instance, "value")?);
        let length = context.get_field(instance, "length")?;
        let name = context
            .load_array(&name_array, 0, length)?
            .into_iter()
            .map(|x| x as u8)
            .collect::<Vec<_>>();

        Ok(str::from_utf8(&name)?.into())
    }
}
