use alloc::vec;

use wie_backend::Image;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::org::kwis::msp::lcdui::Jlet,
    Array,
};

// class org.kwis.msp.lcdui.EventQueue
pub struct EventQueue {}

impl EventQueue {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Jlet;)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("getNextEvent", "([I)V", Self::get_next_event, JavaMethodFlag::NONE),
                JavaMethodProto::new("dispatchEvent", "([I)V", Self::dispatch_event, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<EventQueue>, jlet: JavaObjectProxy<Jlet>) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.EventQueue::<init>({:#x}, {:#x})",
            this.ptr_instance,
            jlet.ptr_instance
        );

        Ok(())
    }

    async fn get_next_event(context: &mut dyn JavaContext, this: JavaObjectProxy<EventQueue>, event: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.EventQueue::getNextEvent({:#x}, {:#x})",
            this.ptr_instance,
            event.ptr_instance
        );

        loop {
            let maybe_event = context.backend().events().pop();

            if maybe_event.is_some() {
                context.store_array_u32(&event, 0, &[41])?; // TODO correct event conversion, 41: REPAINT_EVENT

                break;
            } else {
                context.sleep(16).await; // TODO we need to wait for events
            }
        }

        Ok(())
    }

    async fn dispatch_event(context: &mut dyn JavaContext, this: JavaObjectProxy<EventQueue>, event: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.EventQueue::dispatchEvent({:#x}, {:#x})",
            this.ptr_instance,
            event.ptr_instance
        );

        let event = context.load_array_u32(&event, 0, 4)?;

        match event[0] {
            41 => {
                Self::repaint(context).await?;
            }
            _ => {
                unimplemented!("unhandled event {}", event[0]);
            }
        }

        Ok(())
    }

    async fn repaint(context: &mut dyn JavaContext) -> JavaResult<()> {
        let jlet = JavaObjectProxy::new(
            context
                .call_static_method("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", &[])
                .await?,
        );

        let display = JavaObjectProxy::new(context.get_field(&jlet, "dis")?);
        if display.ptr_instance == 0 {
            return Ok(());
        }

        let cards = JavaObjectProxy::new(context.get_field(&display, "cards")?);
        let card = JavaObjectProxy::new(context.load_array_u32(&cards, 0, 1)?[0]);
        if card.ptr_instance == 0 {
            return Ok(());
        }

        let graphics = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;").await?;
        context
            .call_method(&graphics, "<init>", "(Lorg/kwis/msp/lcdui/Display;)V", &[display.ptr_instance])
            .await?;

        context
            .call_method(&card, "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", &[graphics.ptr_instance])
            .await?;

        let image = JavaObjectProxy::new(context.get_field(&graphics, "img")?);
        context.destroy(graphics)?;

        if image.ptr_instance != 0 {
            let data = JavaObjectProxy::new(context.get_field(&image, "imgData")?);
            let size = context.array_length(&data)?;
            let buffer = context.load_array_u8(&data, 0, size)?;

            context.destroy(data.cast())?;
            context.destroy(image)?;

            let mut canvas = context.backend().screen_canvas();
            let (width, height) = (canvas.width(), canvas.height());

            let src_canvas = Image::from_raw(width, height, buffer);

            canvas.draw(0, 0, width, height, &src_canvas, 0, 0);
            drop(canvas);

            context.backend().repaint();
        }

        Ok(())
    }
}
