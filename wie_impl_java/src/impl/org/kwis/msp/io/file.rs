use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
    r#impl::java::lang::String,
    JavaContext, JavaMethodFlag, JavaResult,
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
                JavaMethodProto::new("write", "([BII)I", Self::write, JavaMethodFlag::NONE),
                JavaMethodProto::new("close", "()V", Self::close, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        filename: JvmClassInstanceProxy<String>,
        mode: i32,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::<init>({:?}, {:?}, {:?})", &this, &filename, mode);

        let filename = String::to_rust_string(context, &filename.class_instance.unwrap())?;
        tracing::debug!("filename: {}", filename);

        Ok(())
    }

    async fn init_with_flag(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        filename: JvmClassInstanceProxy<String>,
        mode: i32,
        flag: i32,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::<init>({:?}, {:?}, {:?}, {:?})", &this, &filename, mode, flag);

        let filename = String::to_rust_string(context, &filename.class_instance.unwrap())?;
        tracing::debug!("filename: {}", filename);

        Ok(())
    }

    async fn write(
        _context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        buf: JvmClassInstanceProxy<JvmClassInstanceProxy<u8>>,
        offset: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.File::write({:?}, {:?}, {:?}, {:?})", &this, &buf, offset, len);

        Ok(len)
    }

    async fn close(_context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::close({:?})", &this);

        Ok(())
    }
}
