use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::Graphics;

use crate::classes::org::kwis::msp::lcdui::Card;

// class wie.CardCanvas
pub struct CardCanvas;

impl CardCanvas {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wie/CardCanvas",
            parent_class: Some("javax/microedition/lcdui/Canvas"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, Default::default()),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, Default::default()),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, Default::default()),
                JavaMethodProto::new("pushCard", "(Lorg/kwis/msp/lcdui/Card;)V", Self::push_card, Default::default()),
                JavaMethodProto::new("removeAllCards", "()V", Self::remove_all_cards, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("cards", "Ljava/util/Vector;", Default::default())],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("wie.CardCanvas::<init>({:?})", this);

        let cards = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(&mut this, "cards", "Ljava/util/Vector;", cards).await?;

        Ok(())
    }

    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("wie.CardCanvas::paint({:?}, {:?})", this, g);

        let graphics = jvm
            .new_class("org/kwis/msp/lcdui/Graphics", "(Ljavax/microedition/lcdui/Graphics;)V", (g,))
            .await?;

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let _: () = jvm
                .invoke_virtual(&card, "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", (graphics.clone(),))
                .await?;
        }

        Ok(())
    }

    async fn key_pressed(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("wie.CardCanvas::keyPressed({:?}, {})", this, key_code);

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let propagate: bool = jvm.invoke_virtual(&card, "keyNotify", "(II)Z", (1i32, key_code)).await?;

            if !propagate {
                break;
            }
        }

        Ok(())
    }

    async fn key_released(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("wie.CardCanvas::keyReleased({:?}, {})", this, key_code);

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let propagate: bool = jvm.invoke_virtual(&card, "keyNotify", "(II)Z", (2i32, key_code)).await?;

            if !propagate {
                break;
            }
        }

        Ok(())
    }

    async fn push_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, c: ClassInstanceRef<Card>) -> JvmResult<()> {
        tracing::debug!("wie.CardCanvas::pushCard({:?}, {:?})", &this, &c);

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let _: () = jvm.invoke_virtual(&cards, "addElement", "(Ljava/lang/Object;)V", (c.clone(),)).await?;

        let _: () = jvm.invoke_virtual(&c, "showNotify", "(Z)V", (true,)).await?;

        Ok(())
    }

    async fn remove_all_cards(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("wie.CardCanvas::removeAllCards");

        let cards = jvm.get_field(&this, "cards", "Ljava/util/Vector;").await?;
        let length = jvm.invoke_virtual(&cards, "size", "()I", ()).await?;

        for i in 0..length {
            let card = jvm.invoke_virtual(&cards, "elementAt", "(I)Ljava/lang/Object;", (i,)).await?;
            let _: () = jvm.invoke_virtual(&card, "showNotify", "(Z)V", (false,)).await?;
        }

        let _: () = jvm.invoke_virtual(&cards, "removeAllElements", "()V", ()).await?;

        Ok(())
    }
}
