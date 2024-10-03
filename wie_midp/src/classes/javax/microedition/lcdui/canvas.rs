use alloc::vec;

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.Canvas
pub struct Canvas;

impl Canvas {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Canvas",
            parent_class: Some("javax/microedition/lcdui/Displayable"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("repaint", "()V", Self::repaint, Default::default()),
                JavaMethodProto::new_abstract("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::<init>({:?})", &this);

        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    async fn get_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getWidth({:?})", &this);

        let display = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let width = jvm.invoke_virtual(&display, "getWidth", "()I", ()).await?;

        Ok(width)
    }

    async fn get_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Canvas::getHeight({:?})", &this);

        let display = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let height = jvm.invoke_virtual(&display, "getHeight", "()I", ()).await?;

        Ok(height)
    }

    async fn repaint(_jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Canvas::repaint({:?})", &this);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

        Ok(())
    }
}
