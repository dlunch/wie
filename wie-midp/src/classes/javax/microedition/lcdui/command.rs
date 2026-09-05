use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

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
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    Self::init_with_long_label,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getLabel", "()Ljava/lang/String;", Self::get_label, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getLongLabel", "()Ljava/lang/String;", Self::get_long_label, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCommandType", "()I", Self::get_command_type, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getPriority", "()I", Self::get_priority, MethodAccessFlags::PUBLIC),
            ],
            fields: ["SCREEN", "BACK", "CANCEL", "OK", "HELP", "STOP", "EXIT", "ITEM"]
                .into_iter()
                .map(|name| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL))
                .chain([
                    JavaFieldProto::new("label", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("longLabel", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("commandType", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("priority", "I", FieldAccessFlags::PRIVATE),
                ])
                .collect(),
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Command::<clinit>");

        for (name, value) in [
            ("SCREEN", 1),
            ("BACK", 2),
            ("CANCEL", 3),
            ("OK", 4),
            ("HELP", 5),
            ("STOP", 6),
            ("EXIT", 7),
            ("ITEM", 8),
        ] {
            jvm.put_static_field("javax/microedition/lcdui/Command", name, "I", value).await?;
        }

        Ok(())
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

        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Command",
            "<init>",
            "(Ljava/lang/String;Ljava/lang/String;II)V",
            (label, ClassInstanceRef::<String>::new(None), command_type, priority),
        )
        .await
    }

    async fn init_with_long_label(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        long_label: ClassInstanceRef<String>,
        command_type: i32,
        priority: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Command::<init>({this:?}, {label:?}, {long_label:?}, {command_type}, {priority})");

        if label.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Command short label is null").await);
        }
        if !(1..=8).contains(&command_type) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid command type").await);
        }

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "label", "Ljava/lang/String;", label).await?;
        jvm.put_field(&mut this, "longLabel", "Ljava/lang/String;", long_label).await?;
        jvm.put_field(&mut this, "commandType", "I", command_type).await?;
        jvm.put_field(&mut this, "priority", "I", priority).await?;

        Ok(())
    }

    async fn get_label(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.Command::getLabel({this:?})");

        let label: ClassInstanceRef<String> = jvm.get_field(&this, "label", "Ljava/lang/String;").await?;

        Ok(label)
    }

    async fn get_long_label(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.Command::getLongLabel({this:?})");

        jvm.get_field(&this, "longLabel", "Ljava/lang/String;").await
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
