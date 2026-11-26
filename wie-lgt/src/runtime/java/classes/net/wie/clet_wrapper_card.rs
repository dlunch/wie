use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_wipi_java::classes::org::kwis::msp::lcdui::Graphics;

use super::CletWrapperContext;

// class net.wie.CletWrapperCard
#[java_class(name = "net/wie/CletWrapperCard", extends = "org/kwis/msp/lcdui/Card", public)]
pub struct CletWrapperCard {
    #[field(private)]
    paintClet: i32,
    #[field(private)]
    handleCletEvent: i32,
}

#[java_methods]
impl CletWrapperCard {
    #[method(constructor, public)]
    async fn init(
        jvm: &Jvm,
        _context: &mut CletWrapperContext,
        mut this: ClassInstanceRef<Self>,
        paint_clet: i32,
        handle_clet_event: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapperCard::<init>({this:?}, {paint_clet:#x}, {handle_clet_event:#x})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "net/wie/CletWrapperCard", "paintClet", "I", paint_clet).await?;
        jvm.put_field(&mut this, "net/wie/CletWrapperCard", "handleCletEvent", "I", handle_clet_event)
            .await?;

        Ok(())
    }

    #[method(protected)]
    async fn paint(
        jvm: &Jvm,
        context: &mut CletWrapperContext,
        this: ClassInstanceRef<Self>,
        _graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapperCard::paint({this:?})");

        let paint_clet: i32 = jvm.get_field(&this, "net/wie/CletWrapperCard", "paintClet", "I").await?;

        context.call_function(jvm, paint_clet as _, &[]).await
    }

    #[method(protected)]
    async fn key_notify(jvm: &Jvm, context: &mut CletWrapperContext, this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("net.wie.CletWrapperCard::keyNotify({this:?}, {type}, {key})");

        let handle_clet_event: i32 = jvm.get_field(&this, "net/wie/CletWrapperCard", "handleCletEvent", "I").await?;
        let r#type = r#type + 501; // TODO constants
        context.call_function(jvm, handle_clet_event as _, &[r#type as _, key as _, 0]).await?;

        Ok(true)
    }

    #[method(public)]
    async fn notify_event(
        jvm: &Jvm,
        context: &mut CletWrapperContext,
        this: ClassInstanceRef<Self>,
        r#type: i32,
        param1: i32,
        param2: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapperCard::notifyEvent({this:?}, {type}, {param1}, {param2})");

        let handle_clet_event: i32 = jvm.get_field(&this, "net/wie/CletWrapperCard", "handleCletEvent", "I").await?;
        context
            .call_function(jvm, handle_clet_event as _, &[r#type as _, param1 as _, param2 as _])
            .await
    }
}
