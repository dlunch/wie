use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::{Object, Runnable, String};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use wie_midp::classes::javax::microedition::{lcdui::Display as MidpDisplay, midlet::MIDlet};

use crate::classes::org::kwis::msp::lcdui::{Card, Jlet, JletEventListener};

// class org.kwis.msp.lcdui.Display
pub struct Display;

impl Display {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Display",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                    Self::init,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;",
                    Self::get_display,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultDisplay",
                    "()Lorg/kwis/msp/lcdui/Display;",
                    Self::get_default_display,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("isDoubleBuffered", "()Z", Self::is_double_buffered, Default::default()),
                JavaMethodProto::new("getDockedCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_docked_card, Default::default()),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card, Default::default()),
                JavaMethodProto::new("removeAllCards", "()V", Self::remove_all_cards, Default::default()),
                JavaMethodProto::new(
                    "addJletEventListener",
                    "(Lorg/kwis/msp/lcdui/JletEventListener;)V",
                    Self::add_jlet_event_listener,
                    Default::default(),
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, Default::default()),
                JavaMethodProto::new(
                    "getGameAction",
                    "(I)I",
                    Self::get_game_action,
                    MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("midpDisplay", "Ljavax/microedition/lcdui/Display;", Default::default()),
                JavaFieldProto::new("cardCanvas", "Lwie/CardCanvas;", Default::default()),
            ],
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        jlet: ClassInstanceRef<Jlet>,
        display_proxy: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::<init>({:?}, {:?}, {:?})", &this, &jlet, &display_proxy);

        let midlet: ClassInstanceRef<MIDlet> = jvm.get_field(&jlet, "wipiMidlet", "Lwie/WIPIMIDlet;").await?;

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Display",
                "getDisplay",
                "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                (midlet,),
            )
            .await?;

        jvm.put_field(&mut this, "midpDisplay", "Ljavax/microedition/lcdui/Display;", midp_display.clone())
            .await?;

        let card_canvas = jvm.new_class("wie/CardCanvas", "()V", ()).await?;
        jvm.put_field(&mut this, "cardCanvas", "Lwie/CardCanvas;", card_canvas.clone()).await?;

        let _: () = jvm
            .invoke_virtual(&midp_display, "setCurrent", "(Ljavax/microedition/lcdui/Displayable;)V", (card_canvas,))
            .await?;

        Ok(())
    }

    async fn get_display(jvm: &Jvm, _: &mut WieJvmContext, str: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDisplay({:?})", &str);

        let jlet = jvm
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;

        let display = jvm.get_field(&jlet, "dis", "Lorg/kwis/msp/lcdui/Display;").await?;

        Ok(display)
    }

    async fn get_default_display(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Display>> {
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

    async fn get_docked_card(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Card>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::getDockedCard");

        Ok(None.into())
    }

    async fn is_double_buffered(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::isDoubleBuffered({:?})", &this);

        Ok(true)
    }

    async fn push_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, c: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::pushCard({:?}, {:?})", &this, &c);

        let card_canvas = jvm.get_field(&this, "cardCanvas", "Lwie/CardCanvas;").await?;
        let _: () = jvm.invoke_virtual(&card_canvas, "pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", (c,)).await?;

        Ok(())
    }

    async fn remove_all_cards(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::removeAllCards({:?})", &this);

        let card_canvas = jvm.get_field(&this, "cardCanvas", "Lwie/CardCanvas;").await?;
        let _: () = jvm.invoke_virtual(&card_canvas, "removeAllCards", "()V", ()).await?;

        Ok(())
    }

    async fn add_jlet_event_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Display>,
        qel: ClassInstanceRef<JletEventListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Display::addJletEventListener({:?}, {:?})", &this, &qel);

        Ok(())
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getWidth({:?})", &this);

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.get_field(&this, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let width: i32 = jvm.invoke_virtual(&midp_display, "getWidth", "()I", ()).await?;

        Ok(width)
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Display::getHeight({:?})", &this);

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.get_field(&this, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let height: i32 = jvm.invoke_virtual(&midp_display, "getHeight", "()I", ()).await?;

        Ok(height)
    }

    async fn call_serially(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r: ClassInstanceRef<Runnable>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Display::callSerially({:?}, {:?})", &this, &r);

        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.get_field(&this, "midpDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        let _: () = jvm.invoke_virtual(&midp_display, "callSerially", "(Ljava/lang/Runnable;)V", (r,)).await?;

        Ok(())
    }

    async fn get_game_action(_: &Jvm, _: &mut WieJvmContext, key: i32) -> JvmResult<i32> {
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
