use alloc::vec;

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lcdui.InputMethodHandler
pub struct InputMethodHandler;

impl InputMethodHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/InputMethodHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init, Default::default()),
                JavaMethodProto::new("setCurrentMode", "(I)Z", Self::set_current_mode, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, constraint: i32) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lcdui.InputMethodHandler::<init>({this:?}, {constraint})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn set_current_mode(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, mode: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::setCurrentMode({this:?}, {mode})");

        Ok(true)
    }
}
