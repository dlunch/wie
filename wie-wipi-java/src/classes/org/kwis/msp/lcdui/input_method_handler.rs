use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::InputMethodListener;

// class org.kwis.msp.lcdui.InputMethodHandler
pub struct InputMethodHandler;

impl InputMethodHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/InputMethodHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setCurrentMode", "(I)Z", Self::set_current_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setInputMethodListener",
                    "(Lorg/kwis/msp/lcdui/InputMethodListener;)V",
                    Self::set_input_method_listener,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
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

    async fn set_input_method_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<InputMethodListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::setInputMethodListener({this:?}, {listener:?})");

        Ok(())
    }
}
