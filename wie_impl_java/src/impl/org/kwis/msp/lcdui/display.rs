use alloc::vec;

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::{JavaObjectProxy, JvmClassInstanceProxy},
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
                JavaMethodProto::new("removeAllCards", "()V", Self::remove_all_cards, JavaMethodFlag::NONE),
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
        this: JvmClassInstanceProxy<Self>,
        jlet: JavaObjectProxy<Jlet>,
        display_proxy: JavaObjectProxy<Object>,
    ) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Display::<init>({:#x}, {:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            jlet.ptr_instance,
            display_proxy.ptr_instance
        );

        let cards = context.instantiate_array("Lorg/kwis/msp/lcdui/Card;", 1).await?;
        let cards = context.array_instance_from_raw(cards.ptr_instance);
        context.jvm().put_field(
            &this.class_instance,
            "cards",
            "[Lorg/kwis/msp/lcdui/Card;",
            JavaValue::Object(Some(cards)),
        )?;

        let screen_canvas = context.backend().screen_canvas();
        let (width, height) = (screen_canvas.width(), screen_canvas.height());
        drop(screen_canvas);

        context.jvm().put_field(&this.class_instance, "m_w", "I", JavaValue::Int(width as _))?;
        context.jvm().put_field(&this.class_instance, "m_h", "I", JavaValue::Int(height as _))?;

        Ok(())
    }

    async fn get_display(context: &mut dyn JavaContext, str: JavaObjectProxy<String>) -> JavaResult<JvmClassInstanceProxy<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({:#x})", str.ptr_instance);

        let jlet = context
            .call_static_method("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", &[])
            .await?;
        let jlet = context.instance_from_raw(jlet);

        let display = context.jvm().get_field(&jlet, "dis", "Lorg/kwis/msp/lcdui/Display;")?;

        Ok(JvmClassInstanceProxy::new(display.as_object_ref().unwrap().clone()))
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

    async fn push_card(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, c: JvmClassInstanceProxy<Card>) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Display::pushCard({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&c.class_instance)
        );

        let cards = context.jvm().get_field(&this.class_instance, "cards", "[Lorg/kwis/msp/lcdui/Card;")?;
        let card = &context.jvm().load_array(cards.as_object_ref().unwrap(), 0, 1)?[0];

        if card.as_object_ref().is_none() {
            let value = JavaValue::Object(Some(c.class_instance.clone()));
            context.jvm().store_array(cards.as_object_ref().unwrap(), 0, &[value])?;
        }

        Ok(())
    }

    async fn remove_all_cards(_: &mut dyn JavaContext) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::removeAllCards");

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
