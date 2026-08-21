use alloc::{string::ToString, vec};

use futures::TryFutureExt;
use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_midp::classes::javax::microedition::lcdui::Display as MidpDisplay;
use wie_wipi_java::classes::org::kwis::msp::lcdui::Display;

use super::CletWrapperContext;

// class net.wie.CletWrapper
pub struct CletWrapper;

impl CletWrapper {
    pub fn as_proto() -> JavaClassProto<CletWrapperContext> {
        JavaClassProto {
            name: "net/wie/CletWrapper",
            parent_class: Some("org/kwis/msp/lcdui/Jlet"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("startApp", "([Ljava/lang/String;)V", Self::start_app, MethodAccessFlags::PROTECTED),
            ],
            fields: vec![
                JavaFieldProto::new("startClet", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                JavaFieldProto::new("pauseClet", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                JavaFieldProto::new("resumeClet", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                JavaFieldProto::new("destroyClet", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                JavaFieldProto::new("paintClet", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
                JavaFieldProto::new("handleCletEvent", "I", FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut CletWrapperContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapper::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lcdui/Jlet", "<init>", "()V", ()).await?;

        Ok(())
    }

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
        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.get_field(&display, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let _: () = jvm
            .invoke_virtual(&midp_display, "javax/microedition/lcdui/Display", "disablePaint", "()V", ())
            .await?;

        context
            .core
            .run_function(start_clet as _, &[])
            .or_else(async move |x| Err(jvm.exception("net/wie/WieError", &x.to_string()).await))
            .await
    }
}
