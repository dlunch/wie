use alloc::{boxed::Box, vec, vec::Vec};
use core::iter;

use java_runtime::classes::java::lang::{Object, Runnable, String};
use java_runtime_base::{
    JavaError, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle, MethodBody,
};
use jvm::{JavaValue, Jvm};

use crate::{
    classes::org::kwis::msp::lcdui::{Card, Jlet, JletEventListener},
    WieClassProto, WieContext,
};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
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
                JavaMethodProto::new("getGameAction", "(I)I", Self::get_game_action, JavaMethodFlag::NATIVE),
            ],
            fields: vec![
                JavaFieldProto::new("cards", "[Lorg/kwis/msp/lcdui/Card;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("szCard", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("m_w", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("m_h", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(
        jvm: &mut Jvm,
        context: &mut WieContext,
        mut this: JvmClassInstanceHandle<Self>,
        jlet: JvmClassInstanceHandle<Jlet>,
        display_proxy: JvmClassInstanceHandle<Object>,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::<init>({:?}, {:?}, {:?})", &this, &jlet, &display_proxy);

        let cards = jvm.instantiate_array("Lorg/kwis/msp/lcdui/Card;", 10).await?;
        jvm.put_field(&mut this, "cards", "[Lorg/kwis/msp/lcdui/Card;", cards)?;
        jvm.put_field(&mut this, "szCard", "I", 0)?;

        let (width, height) = {
            let mut platform = context.system().platform();
            let screen = platform.screen();
            (screen.width(), screen.height())
        };

        jvm.put_field(&mut this, "m_w", "I", width as i32)?;
        jvm.put_field(&mut this, "m_h", "I", height as i32)?;

        Ok(())
    }

    async fn get_display(jvm: &mut Jvm, _: &mut WieContext, str: JvmClassInstanceHandle<String>) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({:?})", &str);

        let jlet = jvm
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;

        let display = jvm.get_field(&jlet, "dis", "Lorg/kwis/msp/lcdui/Display;")?;

        Ok(display)
    }

    async fn get_default_display(jvm: &mut Jvm, _: &mut WieContext) -> JavaResult<JvmClassInstanceHandle<Display>> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getDefaultDisplay");

        let result = jvm
            .invoke_static(
                "org/kwis/msp/lcdui/Display",
                "getDisplay",
                "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                [None.into()],
            )
            .await?;

        Ok(result)
    }

    async fn get_docked_card(_: &mut Jvm, _: &mut WieContext) -> JavaResult<JvmClassInstanceHandle<Card>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDockedCard");

        Ok(None.into())
    }

    async fn push_card(jvm: &mut Jvm, _: &mut WieContext, mut this: JvmClassInstanceHandle<Self>, c: JvmClassInstanceHandle<Card>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::pushCard({:?}, {:?})", &this, &c);

        let mut cards = jvm.get_field(&this, "cards", "[Lorg/kwis/msp/lcdui/Card;")?;
        let card_size: i32 = jvm.get_field(&this, "szCard", "I")?;

        let cards_data = jvm.load_array(&cards, 0, card_size as usize)?;
        let cards_data = cards_data.into_iter().chain(iter::once(c)).collect::<Vec<_>>();

        jvm.store_array(&mut cards, 0, cards_data)?;
        jvm.put_field(&mut this, "szCard", "I", card_size + 1)?;

        Ok(())
    }

    async fn remove_all_cards(jvm: &mut Jvm, _: &mut WieContext, mut this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::removeAllCards");

        jvm.put_field(&mut this, "szCard", "I", 0)?;

        Ok(())
    }

    async fn add_jlet_event_listener(
        _: &mut Jvm,
        _: &mut WieContext,
        this: JvmClassInstanceHandle<Display>,
        qel: JvmClassInstanceHandle<JletEventListener>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::addJletEventListener({:?}, {:?})", &this, &qel);

        Ok(())
    }

    async fn get_width(jvm: &mut Jvm, _: &mut WieContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getWidth({:?})", &this);

        let width: i32 = jvm.get_field(&this, "m_w", "I")?;

        Ok(width)
    }

    async fn get_height(jvm: &mut Jvm, _: &mut WieContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getHeight({:?})", &this);

        let height: i32 = jvm.get_field(&this, "m_h", "I")?;

        Ok(height)
    }

    async fn call_serially(
        _: &mut Jvm,
        context: &mut WieContext,
        this: JvmClassInstanceHandle<Self>,
        r: JvmClassInstanceHandle<Runnable>,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::callSerially({:?}, {:?})", &this, &r);

        // TODO this method have to queue runnable in event queue, but for now we'll spawn new task

        struct SpawnProxy {
            runnable: JvmClassInstanceHandle<Runnable>,
        }

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError, WieContext> for SpawnProxy {
            async fn call(&self, jvm: &mut Jvm, context: &mut WieContext, _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
                let until = context.system().platform().now() + 16; // TODO
                context.system().sleep(until).await;

                jvm.invoke_virtual(&self.runnable, "java/lang/Runnable", "run", "()V", ()).await?;

                Ok(JavaValue::Void)
            }
        }

        context.spawn(Box::new(SpawnProxy { runnable: r }))?;

        Ok(())
    }

    async fn get_game_action(_: &mut Jvm, _: &mut WieContext, key: i32) -> JavaResult<i32> {
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
