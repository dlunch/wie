use alloc::vec;

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JvmArrayClassInstanceProxy,
    JavaFieldAccessFlag,
};

// class java.lang.System
pub struct System {}

impl System {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, JavaMethodFlag::STATIC),
                JavaMethodProto::new("currentTimeMillis", "()J", Self::current_time_millis, JavaMethodFlag::NATIVE),
                JavaMethodProto::new("gc", "()V", Self::gc, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "arraycopy",
                    "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                    Self::arraycopy,
                    JavaMethodFlag::NATIVE,
                ),
            ],
            fields: vec![JavaFieldProto::new("out", "Ljava/io/PrintStream;", JavaFieldAccessFlag::STATIC)],
        }
    }

    async fn cl_init(context: &mut dyn JavaContext) -> JavaResult<()> {
        tracing::debug!("java.lang.System::<clinit>()");

        let out = context.instantiate("Ljava/io/PrintStream;").await?;
        // TODO call constructor with dummy output stream?

        let out = context.instance_from_raw(out.ptr_instance);
        context
            .jvm()
            .put_static_field("java/lang/System", "out", "Ljava/io/PrintStream;", JavaValue::Object(Some(out)))
            .await?;

        Ok(())
    }

    async fn current_time_millis(context: &mut dyn JavaContext) -> JavaResult<i32> {
        tracing::debug!("java.lang.System::currentTimeMillis()");

        Ok(context.backend().time().now().raw() as _)
    }

    async fn gc(_: &mut dyn JavaContext) -> JavaResult<i32> {
        tracing::warn!("stub java.lang.System::gc()");

        Ok(0)
    }

    async fn arraycopy(
        context: &mut dyn JavaContext,
        src: JvmArrayClassInstanceProxy<usize>,
        src_pos: i32,
        dest: JvmArrayClassInstanceProxy<usize>,
        dest_pos: i32,
        length: i32,
    ) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.System::arraycopy({:#x}, {}, {:#x}, {}, {})",
            context.instance_raw(&src.class_instance),
            src_pos,
            context.instance_raw(&dest.class_instance),
            dest_pos,
            length
        );

        // TODO i think we can make it faster
        let src = context.jvm().load_array(&src.class_instance, src_pos as _, length as _)?;
        context.jvm().store_array(&dest.class_instance, dest_pos as _, &src)?;

        Ok(())
    }
}
