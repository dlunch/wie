use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    handle::JvmClassInstanceHandle,
    JavaFieldAccessFlag,
};

// class org.kwis.msp.lcdui.Card
pub struct Card {}

impl Card {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "(I)V", Self::init_1, JavaMethodFlag::NONE),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, JavaMethodFlag::NONE),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, JavaMethodFlag::NONE),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_with_area, JavaMethodFlag::NONE),
                JavaMethodProto::new("repaint", "()V", Self::repaint, JavaMethodFlag::NONE),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("display", "Lorg/kwis/msp/lcdui/Display;", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Card>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::<init>({:?})", &this);

        Ok(())
    }

    async fn init_1(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Card>, a0: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::<init>({:?}, {})", &this, a0);

        Ok(())
    }

    async fn get_width(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Card>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getWidth({:?})", &this);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        let screen_canvas = screen.canvas();

        Ok(screen_canvas.width() as _)
    }

    async fn get_height(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Card>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getHeight({:?})", &this);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        let screen_canvas = screen.canvas();

        Ok(screen_canvas.height() as _)
    }

    async fn repaint(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Card>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Card::repaint({:?})", &this);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw()?;

        Ok(())
    }

    async fn repaint_with_area(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Card>,
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

    async fn service_repaints(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Card>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::serviceRepaints({:?})", &this);

        let mut platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw()?;

        Ok(())
    }
}
