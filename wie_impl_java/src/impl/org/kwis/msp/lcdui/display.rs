use alloc::{boxed::Box, vec};

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    handle::JvmClassInstanceHandle,
    method::MethodBody,
    r#impl::{
        java::lang::{Object, Runnable, String},
        org::kwis::msp::lcdui::{Card, Jlet, JletEventListener},
    },
    JavaError,
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
                    JavaMethodFlag::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultDisplay",
                    "()Lorg/kwis/msp/lcdui/Display;",
                    Self::get_default_display,
                    JavaMethodFlag::STATIC,
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
                JavaMethodProto::new("getWidth", "()I", Self::get_width, JavaMethodFlag::NONE),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, JavaMethodFlag::NONE),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, JavaMethodFlag::NONE),
                JavaMethodProto::new("getGameAction", "(I)I", Self::get_game_action, JavaMethodFlag::STATIC),
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
        this: JvmClassInstanceHandle<Self>,
        jlet: JvmClassInstanceHandle<Jlet>,
        display_proxy: JvmClassInstanceHandle<Object>,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::<init>({:?}, {:?}, {:?})", &this, &jlet, &display_proxy);

        let cards = context.jvm().instantiate_array("Lorg/kwis/msp/lcdui/Card;", 1).await?;
        context.jvm().put_field(&this, "cards", "[Lorg/kwis/msp/lcdui/Card;", cards)?;

        let (width, height) = {
            let mut platform = context.system().platform();
            let screen = platform.screen();
            (screen.width(), screen.height())
        };

        context.jvm().put_field(&this, "m_w", "I", width as i32)?;
        context.jvm().put_field(&this, "m_h", "I", height as i32)?;

        Ok(())
    }

    async fn get_display(context: &mut dyn JavaContext, str: JvmClassInstanceHandle<String>) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({:?})", &str);

        let jlet = context
            .jvm()
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;

        let display = context.jvm().get_field(&jlet, "dis", "Lorg/kwis/msp/lcdui/Display;")?;

        Ok(display)
    }

    async fn get_default_display(context: &mut dyn JavaContext) -> JavaResult<JvmClassInstanceHandle<Display>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getDefaultDisplay");

        let result = context
            .jvm()
            .invoke_static(
                "org/kwis/msp/lcdui/Display",
                "getDisplay",
                "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                [None.into()],
            )
            .await?;

        Ok(result)
    }

    async fn get_docked_card(_: &mut dyn JavaContext) -> JavaResult<JvmClassInstanceHandle<Card>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDockedCard");

        Ok(None.into())
    }

    async fn push_card(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>, c: JvmClassInstanceHandle<Card>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::pushCard({:?}, {:?})", &this, &c);

        let cards = context.jvm().get_field(&this, "cards", "[Lorg/kwis/msp/lcdui/Card;")?;
        let card: &JvmClassInstanceHandle<Card> = &context.jvm().load_array(&cards, 0, 1)?[0];

        if card.is_null() {
            context.jvm().store_array(&cards, 0, [c])?;
        }

        Ok(())
    }

    async fn remove_all_cards(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::removeAllCards");

        let cards = context.jvm().get_field(&this, "cards", "[Lorg/kwis/msp/lcdui/Card;")?;
        context.jvm().store_array(&cards, 0, [None])?;

        Ok(())
    }

    async fn add_jlet_event_listener(
        _: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Display>,
        qel: JvmClassInstanceHandle<JletEventListener>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::addJletEventListener({:?}, {:?})", &this, &qel);

        Ok(())
    }

    async fn get_width(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getWidth({:?})", &this);

        let width: i32 = context.jvm().get_field(&this, "m_w", "I")?;

        Ok(width)
    }

    async fn get_height(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getHeight({:?})", &this);

        let height: i32 = context.jvm().get_field(&this, "m_h", "I")?;

        Ok(height)
    }

    async fn call_serially(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>, r: JvmClassInstanceHandle<Runnable>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::callSerially({:?}, {:?})", &this, &r);

        // TODO this method have to queue runnable in event queue, but for now we'll spawn new task

        struct SpawnProxy {
            runnable: JvmClassInstanceHandle<Runnable>,
        }

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError> for SpawnProxy {
            async fn call(&self, context: &mut dyn JavaContext, _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
                let until = context.system().platform().now() + 8;
                context.system().sleep(until).await;

                context
                    .jvm()
                    .invoke_virtual(&self.runnable, "java/lang/Runnable", "run", "()V", ())
                    .await?;

                Ok(JavaValue::Void)
            }
        }

        context.spawn(Box::new(SpawnProxy { runnable: r }))?;

        Ok(())
    }

    async fn get_game_action(_: &mut dyn JavaContext, key: i32) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getGameAction({})", key);

        let action = match key {
            -1 => 1, // UP
            -2 => 6, // DOWN
            -3 => 2, // LEFT
            -4 => 5, // RIGHT
            -5 => 8, // FIRE,
            _ => 0,
        };

        Ok(action)
    }
}
