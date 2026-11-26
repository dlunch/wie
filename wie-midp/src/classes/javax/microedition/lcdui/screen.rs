use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::Graphics;

// class javax.microedition.lcdui.Screen
#[java_class(
    name = "javax/microedition/lcdui/Screen",
    extends = "javax/microedition/lcdui/Displayable",
    public,
    abstract
)]
pub struct Screen;

#[java_methods]
impl Screen {
    #[method(constructor)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Screen::<init>({this:?})");

        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    #[method]
    async fn handle_paint_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Screen::handlePaintEvent({this:?}, {graphics:?})");

        let width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xffffff,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "fillRect",
                "(IIII)V",
                (0, 0, width, height),
            )
            .await?;
        jvm.invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0,))
            .await
    }
}
