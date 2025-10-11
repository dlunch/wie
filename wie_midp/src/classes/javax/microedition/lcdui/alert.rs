use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::AlertType;

// class javax.microedition.lcdui.Alert
pub struct Alert;

impl Alert {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Alert",
            parent_class: Some("javax/microedition/lcdui/Screen"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("setType", "(Ljavax/microedition/lcdui/AlertType;)V", Self::set_type, Default::default()),
                JavaMethodProto::new("setTimeout", "(I)V", Self::set_timeout, Default::default()),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::<init>({this:?}, {title:?})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn set_type(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        alert_type: ClassInstanceRef<AlertType>,
    ) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Alert::setType({this:?}, {alert_type:?})");

        Ok(())
    }

    async fn set_timeout(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, timeout: i32) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Alert::setTimeout({this:?}, {timeout})");

        Ok(())
    }

    async fn set_string(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Alert::setString({this:?}, {text:?})");

        Ok(())
    }
}
