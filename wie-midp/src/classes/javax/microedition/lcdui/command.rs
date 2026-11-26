use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class javax.microedition.lcdui.Command
#[java_class(name = "javax/microedition/lcdui/Command", public)]
pub struct Command {
    #[field(public, static, final)]
    SCREEN: i32,
    #[field(public, static, final)]
    BACK: i32,
    #[field(public, static, final)]
    CANCEL: i32,
    #[field(public, static, final)]
    OK: i32,
    #[field(public, static, final)]
    HELP: i32,
    #[field(public, static, final)]
    STOP: i32,
    #[field(public, static, final)]
    EXIT: i32,
    #[field(public, static, final)]
    ITEM: i32,
    #[field(private)]
    label: ClassInstanceRef<String>,
    #[field(private)]
    longLabel: ClassInstanceRef<String>,
    #[field(private)]
    commandType: i32,
    #[field(private)]
    priority: i32,
}

#[java_methods]
impl Command {
    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Command::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/Command", "SCREEN", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Command", "BACK", "I", 2).await?;
        jvm.put_static_field("javax/microedition/lcdui/Command", "CANCEL", "I", 3).await?;
        jvm.put_static_field("javax/microedition/lcdui/Command", "OK", "I", 4).await?;
        jvm.put_static_field("javax/microedition/lcdui/Command", "HELP", "I", 5).await?;
        jvm.put_static_field("javax/microedition/lcdui/Command", "STOP", "I", 6).await?;
        jvm.put_static_field("javax/microedition/lcdui/Command", "EXIT", "I", 7).await?;
        jvm.put_static_field("javax/microedition/lcdui/Command", "ITEM", "I", 8).await?;

        Ok(())
    }

    #[method(constructor, public)]
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
            (label, None, command_type, priority),
        )
        .await
    }

    #[method(constructor, public)]
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

        jvm.put_field(&mut this, "javax/microedition/lcdui/Command", "label", "Ljava/lang/String;", label)
            .await?;
        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/Command",
            "longLabel",
            "Ljava/lang/String;",
            long_label,
        )
        .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Command", "commandType", "I", command_type)
            .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Command", "priority", "I", priority)
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn get_label(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.Command::getLabel({this:?})");

        let label: ClassInstanceRef<String> = jvm
            .get_field(&this, "javax/microedition/lcdui/Command", "label", "Ljava/lang/String;")
            .await?;

        Ok(label)
    }

    #[method(public)]
    async fn get_long_label(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.Command::getLongLabel({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/Command", "longLabel", "Ljava/lang/String;")
            .await
    }

    #[method(public)]
    async fn get_command_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Command::getCommandType({this:?})");

        let command_type: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Command", "commandType", "I").await?;

        Ok(command_type)
    }

    #[method(public)]
    async fn get_priority(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Command::getPriority({this:?})");

        let priority: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Command", "priority", "I").await?;

        Ok(priority)
    }
}
