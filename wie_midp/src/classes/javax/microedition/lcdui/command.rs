use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.Command
pub struct Command;

impl Command {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Command",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init, Default::default())],
            fields: vec![],
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        command_type: i32,
        priority: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Command::<init>({this:?}, {label:?}, {command_type}, {priority})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }
}
