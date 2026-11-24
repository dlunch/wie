use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

const CONSTRAINTS: [(&str, i32); 13] = [
    ("ANY", 0),
    ("EMAILADDR", 1),
    ("NUMERIC", 2),
    ("PHONENUMBER", 3),
    ("URL", 4),
    ("DECIMAL", 5),
    ("PASSWORD", 0x10000),
    ("UNEDITABLE", 0x20000),
    ("SENSITIVE", 0x40000),
    ("NON_PREDICTIVE", 0x80000),
    ("INITIAL_CAPS_WORD", 0x100000),
    ("INITIAL_CAPS_SENTENCE", 0x200000),
    ("CONSTRAINT_MASK", 0xffff),
];

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
            fields: CONSTRAINTS
                .iter()
                .map(|(name, _)| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL))
                .collect(),
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextField::<clinit>");

        for (name, value) in CONSTRAINTS {
            jvm.put_static_field("javax/microedition/lcdui/TextField", name, "I", value).await?;
        }

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
