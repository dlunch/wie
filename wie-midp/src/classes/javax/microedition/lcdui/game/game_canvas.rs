use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

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
                JavaMethodProto::new("<init>", "(Z)V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    Self::get_graphics,
                    MethodAccessFlags::PROTECTED,
                ),
                JavaMethodProto::new("flushGraphics", "()V", Self::flush_graphics, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("flushGraphics", "(IIII)V", Self::flush_graphics_region, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getKeyStates", "()I", Self::get_key_states, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::paint,
                    MethodAccessFlags::PROTECTED,
                ),
            ],
            fields: vec![JavaFieldProto::new(
                "offscreenImage",
                "Ljavax/microedition/lcdui/Image;",
                FieldAccessFlags::PRIVATE,
            )],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, suppress_key_events: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.GameCanvas::<init>({this:?}, {suppress_key_events})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await?;

        // The buffer must cover the full screen even when the reported Canvas area is smaller.
        let screen = context.system().platform().screen();
        let (width, height) = (screen.width() as i32, screen.height() as i32);

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/game/GameCanvas",
            "offscreenImage",
            "Ljavax/microedition/lcdui/Image;",
            image,
        )
        .await?;

        Ok(())
    }

    async fn get_graphics(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::getGraphics({this:?})");

        let offscreen_image: ClassInstanceRef<Image> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/game/GameCanvas",
                "offscreenImage",
                "Ljavax/microedition/lcdui/Image;",
            )
            .await?;
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

        // The app may draw the next frame into the buffer as soon as this returns, so the paint cannot stay queued.
        let _: () = jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "repaint", "()V", ()).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
            .await
    }

    async fn flush_graphics_region(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::flushGraphics({this:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (x, y, width, height))
            .await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "serviceRepaints", "()V", ())
            .await
    }

    async fn get_key_states(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::getKeyStates({this:?})");

        let held: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Displayable", "heldGameKeys", "I").await?;
        let pressed: i32 = jvm
            .get_field(&this, "javax/microedition/lcdui/Displayable", "pressedGameKeys", "I")
            .await?;
        // Presses since the last call stay reported once, so a quick tap between two polls is not lost.
        jvm.put_field(&mut this, "javax/microedition/lcdui/Displayable", "pressedGameKeys", "I", 0)
            .await?;

        let shown: bool = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "isShown", "()Z", ())
            .await?;
        Ok(if shown { held | pressed } else { 0 })
    }

    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::paint({this:?}, {g:?})");

        let offscreen_image: ClassInstanceRef<Image> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/game/GameCanvas",
                "offscreenImage",
                "Ljavax/microedition/lcdui/Image;",
            )
            .await?;

        let _: () = jvm
            .invoke_virtual(
                &g,
                "javax/microedition/lcdui/Graphics",
                "drawImage",
                "(Ljavax/microedition/lcdui/Image;III)V",
                (offscreen_image, 0, 0, 0),
            )
            .await?;

        Ok(())
    }
}
