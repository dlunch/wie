use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    r#impl::java::lang::String,
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
};

// class org.kwis.msp.io.File
pub struct File {}

impl File {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init_with_flag, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    pub async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<File>, filename: JavaObjectProxy<String>, mode: i32) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.io.File::<init>({:#x}, {:#x}, {:#x})",
            this.ptr_instance,
            filename.ptr_instance,
            mode
        );

        let filename = String::to_rust_string(context, &filename)?;
        tracing::debug!("filename: {}", filename);

        Ok(())
    }

    pub async fn init_with_flag(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<File>,
        filename: JavaObjectProxy<String>,
        mode: i32,
        flag: i32,
    ) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.io.File::<init>({:#x}, {:#x}, {:#x}, {:#x})",
            this.ptr_instance,
            filename.ptr_instance,
            mode,
            flag
        );

        let filename = String::to_rust_string(context, &filename)?;
        tracing::debug!("filename: {}", filename);

        Ok(())
    }
}
