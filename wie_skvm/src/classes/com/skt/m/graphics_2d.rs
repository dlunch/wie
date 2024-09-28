use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::Graphics;

// class com.skt.m.Graphics2D
pub struct Graphics2D;

impl Graphics2D {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/Graphics2D",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "getGraphics2D",
                "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                Self::get_graphics2d,
                MethodAccessFlags::STATIC,
            )],
            fields: vec![],
        }
    }

    async fn get_graphics2d(_jvm: &Jvm, _context: &mut WieJvmContext, graphics: ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub com.skt.m.Graphics2D::getGraphics2D({:?})", graphics);

        Ok(None.into())
    }
}
