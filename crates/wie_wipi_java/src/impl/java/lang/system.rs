use alloc::vec;

use crate::{
    array::Array,
    base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    JavaObjectProxy,
};

// class java.lang.System
pub struct System {}

impl System {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("currentTimeMillis", "()J", Self::current_time_millis, JavaMethodAccessFlag::NATIVE),
                JavaMethodProto::new("gc", "()V", Self::gc, JavaMethodAccessFlag::NATIVE),
                JavaMethodProto::new(
                    "arraycopy",
                    "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                    Self::arraycopy,
                    JavaMethodAccessFlag::NATIVE,
                ),
            ],
            fields: vec![],
        }
    }

    async fn current_time_millis(context: &mut dyn JavaContext) -> JavaResult<u32> {
        log::trace!("java.lang.System::currentTimeMillis()");

        Ok(context.backend().time().now().raw() as u32)
    }

    async fn gc(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::trace!("java.lang.System::gc()");

        Ok(0)
    }

    async fn arraycopy(
        context: &mut dyn JavaContext,
        src: JavaObjectProxy<Array>,
        src_pos: u32,
        dest: JavaObjectProxy<Array>,
        dest_pos: u32,
        length: u32,
    ) -> JavaResult<()> {
        log::trace!(
            "java.lang.System::arraycopy({:#x}, {}, {:#x}, {}, {})",
            src.ptr_instance,
            src_pos,
            dest.ptr_instance,
            dest_pos,
            length
        );

        let src_data = context.load_array(&src.cast(), src_pos, length)?;
        context.store_array(&dest.cast(), dest_pos, &src_data)?;

        Ok(())
    }
}
