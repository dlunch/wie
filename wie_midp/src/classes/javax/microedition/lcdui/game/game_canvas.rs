use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Graphics, Image};

// class javax.microedition.lcdui.game.GameCanvas
pub struct GameCanvas;

impl GameCanvas {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/game/GameCanvas",
            parent_class: Some("javax/microedition/lcdui/Canvas"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    Self::get_graphics,
                    Default::default(),
                ),
                JavaMethodProto::new("flushGraphics", "()V", Self::flush_graphics, Default::default()),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, Default::default()),
            ],
            fields: vec![JavaFieldProto::new(
                "offscreenImage",
                "Ljavax/microedition/lcdui/Image;",
                Default::default(),
            )],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, suppress_key_events: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.GameCanvas::<init>({this:?}, {suppress_key_events})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await?;

        let width: i32 = jvm.invoke_virtual(&this, "getWidth", "()I", ()).await?;
        let height: i32 = jvm.invoke_virtual(&this, "getHeight", "()I", ()).await?;

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        jvm.put_field(&mut this, "offscreenImage", "Ljavax/microedition/lcdui/Image;", image)
            .await?;

        Ok(())
    }

    async fn get_graphics(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::getGraphics({this:?})");

        let offscreen_image: ClassInstanceRef<Image> = jvm.get_field(&this, "offscreenImage", "Ljavax/microedition/lcdui/Image;").await?;
        let graphics = jvm
            .new_class(
                "javax/microedition/lcdui/Graphics",
                "(Ljavax/microedition/lcdui/Image;)V",
                (offscreen_image,),
            )
            .await?;

        Ok(graphics.into())
    }

    async fn flush_graphics(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::flushGraphics({this:?})");

        let _: () = jvm.invoke_virtual(&this, "repaint", "()V", ()).await?;

        Ok(())
    }

    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::paint({this:?}, {g:?})");

        let offscreen_image: ClassInstanceRef<Image> = jvm.get_field(&this, "offscreenImage", "Ljavax/microedition/lcdui/Image;").await?;

        let _: () = jvm
            .invoke_virtual(&g, "drawImage", "(Ljavax/microedition/lcdui/Image;III)V", (offscreen_image, 0, 0, 0))
            .await?;

        Ok(())
    }
}
