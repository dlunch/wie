use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.TextField
pub struct TextField;

impl TextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/TextField",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "<init>",
                "(Ljava/lang/String;Ljava/lang/String;II)V",
                Self::init,
                MethodAccessFlags::PUBLIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.TextField::<init>({this:?}, {label:?}, {text:?}, {max_size}, {constraints})");

        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await
    }
}
