use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
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
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint, JavaMethodFlag::NONE),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("display", "Lorg/kwis/msp/lcdui/Display;", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Card>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn init_1(_: &mut dyn JavaContext, this: JavaObjectProxy<Card>, a0: u32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::<init>({:#x}, {})", this.ptr_instance, a0);

        Ok(())
    }

    async fn get_width(context: &mut dyn JavaContext, this: JavaObjectProxy<Card>) -> JavaResult<u32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getWidth({:#x})", this.ptr_instance);

        let screen_canvas = context.backend().screen_canvas();

        Ok(screen_canvas.width())
    }

    async fn get_height(context: &mut dyn JavaContext, this: JavaObjectProxy<Card>) -> JavaResult<u32> {
        tracing::debug!("org.kwis.msp.lcdui.Card::getHeight({:#x})", this.ptr_instance);

        let screen_canvas = context.backend().screen_canvas();

        Ok(screen_canvas.height())
    }

    async fn repaint(_: &mut dyn JavaContext, this: JavaObjectProxy<Card>, a0: u32, a1: u32, a2: u32, a3: u32) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Card::repaint({:#x}, {}, {}, {}, {})",
            this.ptr_instance,
            a0,
            a1,
            a2,
            a3
        );

        Ok(())
    }

    async fn service_repaints(_: &mut dyn JavaContext, this: JavaObjectProxy<Card>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Card::serviceRepaints({:#x})", this.ptr_instance);

        Ok(())
    }
}
