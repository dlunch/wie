use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::Runnable;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::{
        lcdui::{Displayable, Graphics, Image},
        midlet::MIDlet,
    },
    net::wie::KeyboardEventType,
};

// class javax.microedition.lcdui.Display
pub struct Display;

impl Display {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Display",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    Self::set_current,
                    Default::default(),
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, Default::default()),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                    Self::get_display,
                    MethodAccessFlags::STATIC,
                ),
                // wie private methods...
                JavaMethodProto::new("handlePaintEvent", "()V", Self::handle_paint_event, Default::default()),
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("currentDisplayable", "Ljavax/microedition/lcdui/Displayable;", Default::default()),
                JavaFieldProto::new("screenImage", "Ljavax/microedition/lcdui/Image;", Default::default()),
                JavaFieldProto::new("screenGraphics", "Ljavax/microedition/lcdui/Graphics;", Default::default()),
                JavaFieldProto::new("width", "I", Default::default()),
                JavaFieldProto::new("height", "I", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let (width, height) = {
            let mut platform = context.system().platform();
            let screen = platform.screen();
            (screen.width() as i32, screen.height() as i32)
        };

        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await?;

        let screen_image = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;
        let screen_graphics: ClassInstanceRef<Graphics> = jvm
            .invoke_virtual(&screen_image, "getGraphics", "()Ljavax/microedition/lcdui/Graphics;", ())
            .await?;

        jvm.put_field(&mut this, "screenImage", "Ljavax/microedition/lcdui/Image;", screen_image)
            .await?;
        jvm.put_field(&mut this, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;", screen_graphics)
            .await?;

        Ok(())
    }

    async fn get_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Display::getWidth({:?})", &this);

        let width = jvm.get_field(&this, "width", "I").await?;

        Ok(width)
    }

    async fn get_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Display::getHeight({:?})", &this);

        let height = jvm.get_field(&this, "height", "I").await?;

        Ok(height)
    }

    async fn call_serially(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Runnable>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::callSerially({:?}, {:?})", &this, &event);

        let event_queue = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(&event_queue, "callSerially", "(Ljava/lang/Runnable;)V", (event,))
            .await?;

        Ok(())
    }

    async fn set_current(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        displayable: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::setCurrent({:?}, {:?})", &this, displayable);

        jvm.put_field(&mut this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;", displayable)
            .await?;

        Ok(())
    }

    async fn get_display(jvm: &Jvm, _context: &mut WieJvmContext, midlet: ClassInstanceRef<MIDlet>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("javax.microedition.lcdui.Display::getDisplay({:?})", midlet);

        let display = jvm.get_field(&midlet, "display", "Ljavax/microedition/lcdui/Display;").await?;

        Ok(display)
    }

    async fn handle_key_event(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Display::handleKeyEvent({:?}, {:?}, {})",
            &this,
            event_type,
            code
        );

        let current_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        if !current_displayable.is_null() && jvm.is_instance(&**current_displayable, "javax/microedition/lcdui/Canvas").await? {
            let event_type = KeyboardEventType::from_raw(event_type);

            match event_type {
                // TODO we need enum
                KeyboardEventType::KeyPressed => jvm.invoke_virtual(&current_displayable, "keyPressed", "(I)V", (code,)).await?,
                KeyboardEventType::KeyReleased => jvm.invoke_virtual(&current_displayable, "keyReleased", "(I)V", (code,)).await?,
                _ => unimplemented!(),
            }
        }

        Ok(())
    }

    async fn handle_paint_event(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::handlePaintEvent({:?})", &this);

        let current_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        if !current_displayable.is_null() && jvm.is_instance(&**current_displayable, "javax/microedition/lcdui/Canvas").await? {
            let screen_graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
            let screen_image: ClassInstanceRef<Image> = jvm.get_field(&this, "screenImage", "Ljavax/microedition/lcdui/Image;").await?;

            let _: () = jvm
                .invoke_virtual(
                    &current_displayable,
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    (screen_graphics,),
                )
                .await?;

            let image = Image::image(jvm, &screen_image).await?;

            let mut platform = context.system().platform();
            let screen = platform.screen();

            screen.paint(&*image);
        }

        Ok(())
    }
}
