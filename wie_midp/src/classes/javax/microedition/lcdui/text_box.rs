use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.TextBox
pub struct TextBox;

impl TextBox {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/TextBox",
            parent_class: Some("javax/microedition/lcdui/Screen"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "<init>",
                "(Ljava/lang/String;Ljava/lang/String;II)V",
                Self::init,
                Default::default(),
            )],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::<init>({this:?}, {title:?}, {text:?}, {max_size}, {constraints})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await?;

        Ok(())
    }
}
