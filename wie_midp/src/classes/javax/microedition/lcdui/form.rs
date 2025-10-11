use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::Item;

// class javax.microedition.lcdui.Form
pub struct Form;

impl Form {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Form",
            parent_class: Some("javax/microedition/lcdui/Screen"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("append", "(Ljavax/microedition/lcdui/Item;)I", Self::append, Default::default()),
                JavaMethodProto::new("append", "(Ljava/lang/String;)I", Self::append_string, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Form::<init>({this:?}, {title:?})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn append(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, item: ClassInstanceRef<Item>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Form::append({this:?}, {item:?})");

        Ok(0)
    }

    async fn append_string(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, str: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Form::append({this:?}, {str:?})");

        Ok(0)
    }
}
