use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
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
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;II)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("ANY", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL),
                JavaFieldProto::new(
                    "EMAILADDR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "NUMERIC",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "PHONENUMBER",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("URL", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL),
                JavaFieldProto::new(
                    "DECIMAL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "PASSWORD",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "UNEDITABLE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SENSITIVE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "NON_PREDICTIVE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "INITIAL_CAPS_WORD",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "INITIAL_CAPS_SENTENCE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "CONSTRAINT_MASK",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/TextField", "ANY", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "EMAILADDR", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "NUMERIC", "I", 2).await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "PHONENUMBER", "I", 3).await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "URL", "I", 4).await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "DECIMAL", "I", 5).await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "PASSWORD", "I", 0x10000)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "UNEDITABLE", "I", 0x20000)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "SENSITIVE", "I", 0x40000)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "NON_PREDICTIVE", "I", 0x80000)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "INITIAL_CAPS_WORD", "I", 0x100000)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "INITIAL_CAPS_SENTENCE", "I", 0x200000)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/TextField", "CONSTRAINT_MASK", "I", 0xffff)
            .await?;

        Ok(())
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
