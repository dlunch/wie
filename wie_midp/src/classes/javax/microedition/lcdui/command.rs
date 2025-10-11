use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
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
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init, Default::default()),
                JavaMethodProto::new("getLabel", "()Ljava/lang/String;", Self::get_label, Default::default()),
                JavaMethodProto::new("getCommandType", "()I", Self::get_command_type, Default::default()),
                JavaMethodProto::new("getPriority", "()I", Self::get_priority, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("label", "Ljava/lang/String;", Default::default()),
                JavaFieldProto::new("commandType", "I", Default::default()),
                JavaFieldProto::new("priority", "I", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        command_type: i32,
        priority: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Command::<init>({this:?}, {label:?}, {command_type}, {priority})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "label", "Ljava/lang/String;", label).await?;
        jvm.put_field(&mut this, "commandType", "I", command_type).await?;
        jvm.put_field(&mut this, "priority", "I", priority).await?;

        Ok(())
    }

    async fn get_label(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.Command::getLabel({this:?})");

        let label: ClassInstanceRef<String> = jvm.get_field(&this, "label", "Ljava/lang/String;").await?;

        Ok(label)
    }

    async fn get_command_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Command::getCommandType({this:?})");

        let command_type: i32 = jvm.get_field(&this, "commandType", "I").await?;

        Ok(command_type)
    }

    async fn get_priority(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Command::getPriority({this:?})");

        let priority: i32 = jvm.get_field(&this, "priority", "I").await?;

        Ok(priority)
    }
}
