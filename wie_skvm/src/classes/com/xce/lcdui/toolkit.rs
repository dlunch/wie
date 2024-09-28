use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.lcdui.Toolkit
pub struct Toolkit;

impl Toolkit {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/Toolkit",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC)],
            fields: vec![JavaFieldProto::new(
                "graphics",
                "Ljavax/microedition/lcdui/Graphics;",
                FieldAccessFlags::STATIC,
            )],
        }
    }

    async fn cl_init(_jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::<clinit>");

        Ok(())
    }
}
