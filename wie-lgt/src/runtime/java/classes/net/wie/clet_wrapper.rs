use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_midp::classes::javax::microedition::lcdui::Display as MidpDisplay;
use wie_wipi_java::classes::org::kwis::msp::lcdui::Display;

use super::CletWrapperContext;

// class net.wie.CletWrapper
#[java_class(name = "net/wie/CletWrapper", extends = "org/kwis/msp/lcdui/Jlet", public)]
pub struct CletWrapper {
    #[field(private, static)]
    startClet: i32,
    #[field(private, static)]
    pauseClet: i32,
    #[field(private, static)]
    resumeClet: i32,
    #[field(private, static)]
    destroyClet: i32,
    #[field(private, static)]
    paintClet: i32,
    #[field(private, static)]
    handleCletEvent: i32,
}

#[java_methods]
impl CletWrapper {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut CletWrapperContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapper::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lcdui/Jlet", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[method(protected)]
    async fn start_app(
        jvm: &Jvm,
        context: &mut CletWrapperContext,
        this: ClassInstanceRef<Self>,
        args: ClassInstanceRef<Array<String>>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapper::startApp({this:?}, {args:?})");

        let start_clet: i32 = jvm.get_static_field("net/wie/CletWrapper", "startClet", "I").await?;
        let paint_clet: i32 = jvm.get_static_field("net/wie/CletWrapper", "paintClet", "I").await?;
        let handle_clet_event: i32 = jvm.get_static_field("net/wie/CletWrapper", "handleCletEvent", "I").await?;

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
            .await?;
        let clet_wrapper_card = jvm.new_class("net/wie/CletWrapperCard", "(II)V", (paint_clet, handle_clet_event)).await?;
        let _: () = jvm
            .invoke_virtual(
                &display,
                "org/kwis/msp/lcdui/Display",
                "pushCard",
                "(Lorg/kwis/msp/lcdui/Card;)V",
                (clet_wrapper_card,),
            )
            .await?;
        let midp_display: ClassInstanceRef<MidpDisplay> = jvm
            .get_field(
                &display,
                "org/kwis/msp/lcdui/Display",
                "midpDisplay",
                "Ljavax/microedition/lcdui/Display;",
            )
            .await?;
        let _: () = jvm
            .invoke_virtual(&midp_display, "javax/microedition/lcdui/Display", "disablePaint", "()V", ())
            .await?;

        context.call_function(jvm, start_clet as _, &[]).await
    }
}
