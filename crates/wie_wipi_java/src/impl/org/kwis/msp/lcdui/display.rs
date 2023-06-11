use alloc::vec;
use wie_base::CanvasHandle;

use crate::{
    base::{JavaAccessFlag, JavaClassProto, JavaContext, JavaFieldProto, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("getDisplay", "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;", Self::get_display),
                JavaMethodProto::new("getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", Self::get_default_display),
                JavaMethodProto::new("getDockedCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_docked_card),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card),
                JavaMethodProto::new(
                    "addJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::add_jlet_event_listener,
                ),
                // private
                JavaMethodProto::new("paint", "()V", Self::paint),
            ],
            fields: vec![
                JavaFieldProto::new("card", "Lorg/kwis/msp/lcdui/Card;", JavaAccessFlag::NONE),
                JavaFieldProto::new("display", "Lorg/kwis/msp/lcdui/Display;", JavaAccessFlag::STATIC),
            ],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub Display::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn get_display(context: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Display::getDisplay({:#x})", a0.ptr_instance);

        let display = context.get_static_field("org/kwis/msp/lcdui/Display", "display")?;
        if display == 0 {
            let instance = context.instantiate("Lorg/kwis/msp/lcdui/Display;")?;
            context.call_method(&instance, "<init>", "()V", &[]).await?;

            context.put_static_field("org/kwis/msp/lcdui/Display", "display", instance.ptr_instance)?;

            Ok(instance)
        } else {
            Ok(JavaObjectProxy::new(display))
        }
    }

    async fn get_default_display(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Display::getDefaultDisplay");

        let ptr_instance = context
            .call_static_method(
                "org/kwis/msp/lcdui/Display",
                "getDisplay",
                "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                &[0],
            )
            .await?;

        Ok(JavaObjectProxy::new(ptr_instance))
    }

    async fn get_docked_card(_: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Display::getDockedCard");

        Ok(JavaObjectProxy::new(0))
    }

    async fn push_card(context: &mut dyn JavaContext, instance: JavaObjectProxy, a1: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub Display::pushCard({:#x}, {:#x})", instance.ptr_instance, a1.ptr_instance);

        let card = context.get_field(&instance, "card")?;
        if card == 0 {
            context.put_field(&instance, "card", a1.ptr_instance)?;
        }

        Ok(())
    }

    async fn add_jlet_event_listener(_: &mut dyn JavaContext, a0: JavaObjectProxy) -> JavaResult<()> {
        log::warn!("stub Display::addJletEventListener({:#x})", a0.ptr_instance);

        Ok(())
    }

    async fn paint(context: &mut dyn JavaContext, canvas_handle: CanvasHandle) -> JavaResult<()> {
        log::trace!("Display::paint");

        let display = context.get_static_field("org/kwis/msp/lcdui/Display", "display")?;
        if display == 0 {
            return Ok(());
        }

        let card = context.get_field(&JavaObjectProxy::new(display), "card")?;
        if card == 0 {
            return Ok(());
        }

        let graphics = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;")?;
        context.call_method(&graphics, "<init>", "(I)V", &[canvas_handle]).await?;

        context
            .call_method(
                &JavaObjectProxy::new(card),
                "paint",
                "(Lorg/kwis/msp/lcdui/Graphics;)V",
                &[graphics.ptr_instance],
            )
            .await?;

        context.destroy(graphics)?;

        Ok(())
    }
}
