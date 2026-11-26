use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::Graphics;

// abstract class javax.microedition.lcdui.game.Layer
#[java_class(name = "javax/microedition/lcdui/game/Layer", public, abstract)]
pub struct Layer {
    #[field(private)]
    x: i32,
    #[field(private)]
    y: i32,
    #[field(private)]
    width: i32,
    #[field(private)]
    height: i32,
    #[field(private)]
    visible: bool,
}

#[java_methods]
impl Layer {
    #[method(public)]
    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()>;

    #[method(constructor)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/game/Layer", "visible", "Z", true)
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn set_position(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::setPosition({this:?}, {x}, {y})");

        jvm.put_field(&mut this, "javax/microedition/lcdui/game/Layer", "x", "I", x).await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/game/Layer", "y", "I", y).await?;

        Ok(())
    }

    #[method(public)]
    async fn r#move(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, dx: i32, dy: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::move({this:?}, {dx}, {dy})");

        let x: i32 = jvm.get_field(&this, "javax/microedition/lcdui/game/Layer", "x", "I").await?;
        let y: i32 = jvm.get_field(&this, "javax/microedition/lcdui/game/Layer", "y", "I").await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/game/Layer", "x", "I", x.wrapping_add(dx))
            .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/game/Layer", "y", "I", y.wrapping_add(dy))
            .await?;

        Ok(())
    }

    #[method(public, final)]
    async fn get_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::getX({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/game/Layer", "x", "I").await
    }

    #[method(public, final)]
    async fn get_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::getY({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/game/Layer", "y", "I").await
    }

    #[method(public, final)]
    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::getWidth({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/game/Layer", "width", "I").await
    }

    #[method(public, final)]
    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::getHeight({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/game/Layer", "height", "I").await
    }

    #[method(public)]
    async fn set_visible(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, visible: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::setVisible({this:?}, {visible})");

        jvm.put_field(&mut this, "javax/microedition/lcdui/game/Layer", "visible", "Z", visible)
            .await
    }

    #[method(public, final)]
    async fn is_visible(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.game.Layer::isVisible({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/game/Layer", "visible", "Z").await
    }
}
