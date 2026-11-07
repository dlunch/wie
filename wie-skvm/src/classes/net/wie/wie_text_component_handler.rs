use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class net.wie.WieTextComponentHandler
pub struct WieTextComponentHandler;

impl WieTextComponentHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WieTextComponentHandler",
            parent_class: Some("com/xce/lcdui/TextComponentHandler"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC)],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieTextComponentHandler::<init>({this:?})");
        jvm.invoke_special(&this, "com/xce/lcdui/TextComponentHandler", "<init>", "()V", ()).await
    }
}
