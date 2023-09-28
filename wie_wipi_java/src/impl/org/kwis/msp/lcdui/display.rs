use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::{
        java::lang::{Object, String},
        org::kwis::msp::lcdui::{Card, Jlet, JletEventListener},
    },
};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                    Self::init,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                    Self::get_display,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new(
                    "getDefaultDisplay",
                    "()Lorg/kwis/msp/lcdui/Display;",
                    Self::get_default_display,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new(
                    "getDockedCard",
                    "()Lorg/kwis/msp/lcdui/Card;",
                    Self::get_docked_card,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "addJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::add_jlet_event_listener,
                    JavaMethodFlag::NONE,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("cards", "[Lorg/kwis/msp/lcdui/Card;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("m_w", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("m_h", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<Display>,
        jlet: JavaObjectProxy<Jlet>,
        display_proxy: JavaObjectProxy<Object>,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Display::<init>({:#x}, {:#x}, {:#x})",
            this.ptr_instance,
            jlet.ptr_instance,
            display_proxy.ptr_instance
        );

        let cards = context.instantiate_array("Lorg/kwis/msp/lcdui/Card;", 1).await?;
        context.put_field(&this.cast(), "cards", cards.ptr_instance)?;

        let screen_canvas = context.backend().screen_canvas();
        let (width, height) = (screen_canvas.width(), screen_canvas.height());
        drop(screen_canvas);

        context.put_field(&this.cast(), "m_w", width as _)?;
        context.put_field(&this.cast(), "m_h", height as _)?;

        Ok(())
    }

    async fn get_display(context: &mut dyn JavaContext, str: JavaObjectProxy<String>) -> JavaResult<JavaObjectProxy<Display>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({:#x})", str.ptr_instance);

        let jlet = JavaObjectProxy::new(
            context
                .call_static_method("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", &[])
                .await?,
        );

        let display = JavaObjectProxy::new(context.get_field(&jlet, "dis")?);

        Ok(display)
    }

    async fn get_default_display(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Display>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getDefaultDisplay");

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

    async fn get_docked_card(_: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Card>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDockedCard");

        Ok(JavaObjectProxy::new(0))
    }

    async fn push_card(context: &mut dyn JavaContext, this: JavaObjectProxy<Display>, c: JavaObjectProxy<Card>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::pushCard({:#x}, {:#x})", this.ptr_instance, c.ptr_instance);

        let cards = JavaObjectProxy::new(context.get_field(&this.cast(), "cards")?);
        let card = context.load_array_i32(&cards, 0, 1)?[0];

        if card == 0 {
            context.store_array_i32(&cards, 0, &[c.ptr_instance as _])?; // TODO store_array_reference
        }

        Ok(())
    }

    async fn add_jlet_event_listener(
        _: &mut dyn JavaContext,
        this: JavaObjectProxy<Display>,
        qel: JavaObjectProxy<JletEventListener>,
    ) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.lcdui.Display::addJletEventListener({:#x}, {:#x})",
            this.ptr_instance,
            qel.ptr_instance
        );

        Ok(())
    }
}
