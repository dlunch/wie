use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::Display;

// class javax.microedition.lcdui.AlertType
#[java_class(name = "javax/microedition/lcdui/AlertType", public)]
pub struct AlertType {
    #[field(public, static, final)]
    ALARM: ClassInstanceRef<Self>,
    #[field(public, static, final)]
    CONFIRMATION: ClassInstanceRef<Self>,
    #[field(public, static, final)]
    ERROR: ClassInstanceRef<Self>,
    #[field(public, static, final)]
    INFO: ClassInstanceRef<Self>,
    #[field(public, static, final)]
    WARNING: ClassInstanceRef<Self>,
}

#[java_methods]
impl AlertType {
    #[method(descriptor = "(Ljava/lang/String;)V", constructor, private)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::<init>({this:?}, {type})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[method(public)]
    async fn play_sound(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        target: ClassInstanceRef<Display>,
    ) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.AlertType::playSound({this:?}, {target:?})");

        Ok(false)
    }

    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.AlertType::<clinit>()");

        jvm.put_static_field(
            "javax/microedition/lcdui/AlertType",
            "ALARM",
            "Ljavax/microedition/lcdui/AlertType;",
            jvm.new_class("javax/microedition/lcdui/AlertType", "(Ljava/lang/String;)V", (0,)).await?,
        )
        .await?;

        jvm.put_static_field(
            "javax/microedition/lcdui/AlertType",
            "CONFIRMATION",
            "Ljavax/microedition/lcdui/AlertType;",
            jvm.new_class("javax/microedition/lcdui/AlertType", "(Ljava/lang/String;)V", (1,)).await?,
        )
        .await?;

        jvm.put_static_field(
            "javax/microedition/lcdui/AlertType",
            "ERROR",
            "Ljavax/microedition/lcdui/AlertType;",
            jvm.new_class("javax/microedition/lcdui/AlertType", "(Ljava/lang/String;)V", (2,)).await?,
        )
        .await?;

        jvm.put_static_field(
            "javax/microedition/lcdui/AlertType",
            "INFO",
            "Ljavax/microedition/lcdui/AlertType;",
            jvm.new_class("javax/microedition/lcdui/AlertType", "(Ljava/lang/String;)V", (3,)).await?,
        )
        .await?;

        jvm.put_static_field(
            "javax/microedition/lcdui/AlertType",
            "WARNING",
            "Ljavax/microedition/lcdui/AlertType;",
            jvm.new_class("javax/microedition/lcdui/AlertType", "(Ljava/lang/String;)V", (4,)).await?,
        )
        .await?;

        Ok(())
    }
}
