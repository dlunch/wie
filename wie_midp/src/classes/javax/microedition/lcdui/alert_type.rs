use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.AlertType
pub struct AlertType;

impl AlertType {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/AlertType",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "ALARM",
                    "Ljavax/microedition/lcdui/AlertType;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "CONFIRMATION",
                    "Ljavax/microedition/lcdui/AlertType;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "ERROR",
                    "Ljavax/microedition/lcdui/AlertType;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "INFO",
                    "Ljavax/microedition/lcdui/AlertType;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "WARNING",
                    "Ljavax/microedition/lcdui/AlertType;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::<init>({this:?}, {type})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

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
