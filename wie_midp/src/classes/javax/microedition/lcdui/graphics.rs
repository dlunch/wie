use alloc::vec;

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.Graphics
pub struct Graphics;

impl Graphics {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Graphics",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("setFont", "(Ljavax/microedition/lcdui/Font;)V", Self::set_font, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn set_font(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, font: ClassInstanceRef<super::Font>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Graphics::setFont({:?}, {:?})", &this, font);

        Ok(())
    }
}
