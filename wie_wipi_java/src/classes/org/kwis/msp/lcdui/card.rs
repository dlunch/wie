use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto, JavaResult};
use jvm::{ClassInstanceRef, Jvm};

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.lcdui.Card
pub struct Card {}

impl Card {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(I)V", Self::init_1, Default::default()),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_with_area, Default::default()),
                JavaMethodProto::new("repaint", "()V", Self::repaint, Default::default()),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("display", "Lorg/kwis/msp/lcdui/Display;", Default::default()),
                JavaFieldProto::new("w", "I", Default::default()),
                JavaFieldProto::new("h", "I", Default::default()),
            ],
        }
    }

    async fn init(jvm: &mut Jvm, _: &mut WIPIJavaContext, mut this: ClassInstanceRef<Card>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::<init>({:?})", &this);

        let display = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;

        let width: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", []).await?;
        let height: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getHeight", "()I", []).await?;

        jvm.put_field(&mut this, "display", "Lorg/kwis/msp/lcdui/Display;", display)?;
        jvm.put_field(&mut this, "w", "I", width)?;
        jvm.put_field(&mut this, "h", "I", height)?;

        Ok(())
    }

    async fn init_1(_: &mut Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>, a0: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::<init>({:?}, {})", &this, a0);

        Ok(())
    }

    async fn get_width(jvm: &mut Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getWidth({:?})", &this);

        jvm.get_field(&this, "w", "I")
    }

    async fn get_height(jvm: &mut Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getHeight({:?})", &this);

        jvm.get_field(&this, "h", "I")
    }

    async fn repaint(jvm: &mut Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::repaint({:?})", &this);

        let width: i32 = jvm.get_field(&this, "w", "I")?;
        let height: i32 = jvm.get_field(&this, "h", "I")?;

        jvm.invoke_virtual(&this, "org/kwis/msp/lcdui/Card", "repaint", "(IIII)V", (0, 0, width, height))
            .await?;

        Ok(())
    }

    async fn repaint_with_area(
        _: &mut Jvm,
        context: &mut WIPIJavaContext,
        this: ClassInstanceRef<Card>,
        a0: i32,
        a1: i32,
        a2: i32,
        a3: i32,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::repaint({:?}, {}, {}, {}, {})", &this, a0, a1, a2, a3);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw()?;

        Ok(())
    }

    async fn service_repaints(_: &mut Jvm, context: &mut WIPIJavaContext, this: ClassInstanceRef<Card>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::serviceRepaints({:?})", &this);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw()?;

        Ok(())
    }
}
