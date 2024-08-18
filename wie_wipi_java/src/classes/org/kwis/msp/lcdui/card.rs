use alloc::{format, vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use crate::{
    classes::org::kwis::msp::lcdui::Display,
    context::{WIPIJavaClassProto, WIPIJavaContext},
};

// class org.kwis.msp.lcdui.Card
pub struct Card {}

impl Card {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            name: "org/kwis/msp/lcdui/Card",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(I)V", Self::init_int, Default::default()),
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Display;)V", Self::init_with_display, Default::default()),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("isShown", "()Z", Self::is_shown, Default::default()),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_with_area, Default::default()),
                JavaMethodProto::new("repaint", "()V", Self::repaint, Default::default()),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("display", "Lorg/kwis/msp/lcdui/Display;", Default::default()),
                JavaFieldProto::new("x", "I", Default::default()),
                JavaFieldProto::new("y", "I", Default::default()),
                JavaFieldProto::new("w", "I", Default::default()),
                JavaFieldProto::new("h", "I", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lcdui.Card::<init>({:?})", &this);

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "(Lorg/kwis/msp/lcdui/Display;)V", (display,))
            .await?;

        Ok(())
    }

    // not in reference, but called by some clet
    async fn init_int(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>, a0: i32) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lcdui.Card::<init>({:?}, {})", &this, a0);

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "(Lorg/kwis/msp/lcdui/Display;)V", (display,))
            .await?;

        Ok(())
    }

    async fn init_with_display(
        jvm: &Jvm,
        _: &mut WIPIJavaContext,
        mut this: ClassInstanceRef<Card>,
        display: ClassInstanceRef<Display>,
    ) -> JvmResult<()> {
        let log = format!("org.kwis.msp.lcdui.Card::<init>({:?}, {:?})", &this, &display);
        tracing::debug!("{}", log); // splitted format as tracing macro doesn't like variable named `display` https://github.com/tokio-rs/tracing/issues/2332

        let width: i32 = jvm.invoke_virtual(&display, "getWidth", "()I", []).await?;
        let height: i32 = jvm.invoke_virtual(&display, "getHeight", "()I", []).await?;

        jvm.put_field(&mut this, "display", "Lorg/kwis/msp/lcdui/Display;", display).await?;
        jvm.put_field(&mut this, "x", "I", 0).await?;
        jvm.put_field(&mut this, "y", "I", 0).await?;
        jvm.put_field(&mut this, "w", "I", width).await?;
        jvm.put_field(&mut this, "h", "I", height).await?;

        Ok(())
    }

    async fn is_shown(_: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::isShown({:?})", &this);

        Ok(true)
    }

    async fn get_width(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getWidth({:?})", &this);

        jvm.get_field(&this, "w", "I").await
    }

    async fn get_height(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getHeight({:?})", &this);

        jvm.get_field(&this, "h", "I").await
    }

    async fn repaint(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::repaint({:?})", &this);

        let width: i32 = jvm.get_field(&this, "w", "I").await?;
        let height: i32 = jvm.get_field(&this, "h", "I").await?;

        let _: () = jvm.invoke_virtual(&this, "repaint", "(IIII)V", (0, 0, width, height)).await?;

        Ok(())
    }

    async fn repaint_with_area(
        _: &Jvm,
        context: &mut WIPIJavaContext,
        this: ClassInstanceRef<Card>,
        a0: i32,
        a1: i32,
        a2: i32,
        a3: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::repaint({:?}, {}, {}, {}, {})", &this, a0, a1, a2, a3);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

        Ok(())
    }

    async fn service_repaints(_: &Jvm, context: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::serviceRepaints({:?})", &this);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

        Ok(())
    }
}
