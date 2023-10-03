use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    r#impl::java::lang::String,
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
};

// class org.kwis.msp.io.FileSystem
pub struct FileSystem {}

impl FileSystem {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "isFile",
                "(Ljava/lang/String;)Z",
                Self::is_file,
                JavaMethodFlag::STATIC,
            )],
            fields: vec![],
        }
    }

    pub async fn is_file(_context: &mut dyn JavaContext, name: JavaObjectProxy<String>, flag: i32) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::is_file({:#x}, {:#x})", name.ptr_instance, flag);

        Ok(0)
    }
}
