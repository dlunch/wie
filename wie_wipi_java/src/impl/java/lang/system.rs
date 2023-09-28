use alloc::vec;

use crate::{
    array::Array,
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
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
                JavaMethodProto::new("currentTimeMillis", "()J", Self::current_time_millis, JavaMethodFlag::NATIVE),
                JavaMethodProto::new("gc", "()V", Self::gc, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "arraycopy",
                    "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                    Self::arraycopy,
                    JavaMethodFlag::NATIVE,
                ),
            ],
            fields: vec![],
        }
    }

    async fn current_time_millis(context: &mut dyn JavaContext) -> JavaResult<i32> {
        tracing::debug!("java.lang.System::currentTimeMillis()");

        Ok(context.backend().time().now().raw() as _)
    }

    async fn gc(_: &mut dyn JavaContext) -> JavaResult<i32> {
        tracing::debug!("java.lang.System::gc()");

        Ok(0)
    }

    async fn arraycopy(
        context: &mut dyn JavaContext,
        src: JavaObjectProxy<Array>,
        src_pos: i32,
        dest: JavaObjectProxy<Array>,
        dest_pos: i32,
        length: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.System::arraycopy({:#x}, {}, {:#x}, {}, {})",
            src.ptr_instance,
            src_pos,
            dest.ptr_instance,
            dest_pos,
            length
        );

        let element_size = context.array_element_size(&src)?;
        match element_size {
            1 => {
                let src_data = context.load_array_i8(&src.cast(), src_pos as _, length as _)?;
                context.store_array_i8(&dest.cast(), dest_pos as _, &src_data)?;
            }
            4 => {
                let src_data = context.load_array_i32(&src.cast(), src_pos as _, length as _)?;
                context.store_array_i32(&dest.cast(), dest_pos as _, &src_data)?;
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
}
