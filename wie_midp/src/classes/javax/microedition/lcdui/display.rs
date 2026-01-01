use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::Runnable;
use jvm::{ClassInstanceRef, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::{
    lcdui::{Displayable, Graphics, Image},
    midlet::MIDlet,
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
                JavaMethodProto::new(
                    "getCurrent",
                    "()Ljavax/microedition/lcdui/Displayable;",
                    Self::get_current,
                    Default::default(),
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, Default::default()),
                JavaMethodProto::new("vibrate", "(I)Z", Self::vibrate, Default::default()),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                    Self::get_display,
                    MethodAccessFlags::STATIC,
                ),
                // wie private methods...
                JavaMethodProto::new("handlePaintEvent", "()V", Self::handle_paint_event, Default::default()),
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, Default::default()),
                JavaMethodProto::new("handleNotifyEvent", "(III)V", Self::handle_notify_event, Default::default()),
                JavaMethodProto::new("setFullscreen", "(Z)V", Self::set_fullscreen, Default::default()),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint, Default::default()),
                JavaMethodProto::new("disablePaint", "()V", Self::disable_paint, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("isInFullScreenMode", "Z", Default::default()),
                JavaFieldProto::new("currentDisplayable", "Ljavax/microedition/lcdui/Displayable;", Default::default()),
                JavaFieldProto::new("screenImage", "Ljavax/microedition/lcdui/Image;", Default::default()),
                JavaFieldProto::new("screenGraphics", "Ljavax/microedition/lcdui/Graphics;", Default::default()),
                JavaFieldProto::new("width", "I", Default::default()),
                JavaFieldProto::new("height", "I", Default::default()),
                JavaFieldProto::new("paintDisabled", "Z", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let (width, height) = {
            let platform = context.system().platform();
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

        let old_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        if !old_displayable.is_null() {
            let _: () = jvm
                .invoke_virtual(&old_displayable, "setDisplay", "(Ljavax/microedition/lcdui/Display;)V", (None,))
                .await?;
        }

        jvm.put_field(
            &mut this,
            "currentDisplayable",
            "Ljavax/microedition/lcdui/Displayable;",
            displayable.clone(),
        )
        .await?;

        let _: () = jvm
            .invoke_virtual(&displayable, "setDisplay", "(Ljavax/microedition/lcdui/Display;)V", (this.clone(),))
            .await?;

        let fullscreen_mode: bool = jvm.get_field(&displayable, "isInFullScreenMode", "Z").await?;
        jvm.put_field(&mut this, "isInFullScreenMode", "Z", fullscreen_mode).await?;

        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;

        let _: () = jvm.invoke_virtual(&this, "repaint", "(IIII)V", (0, 0, width, height)).await?;

        Ok(())
    }

    async fn get_current(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Displayable>> {
        tracing::debug!("javax.microedition.lcdui.Display::getCurrent({this:?})");

        let current_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        Ok(current_displayable)
    }

    async fn get_display(jvm: &Jvm, _context: &mut WieJvmContext, midlet: ClassInstanceRef<MIDlet>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("javax.microedition.lcdui.Display::getDisplay({:?})", midlet);

        let display = MIDlet::display(jvm, &midlet).await?;

        Ok(display)
    }

    async fn vibrate(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, duration: i32) -> JvmResult<bool> {
        tracing::warn!("stub javax.microedition.lcdui.Display::vibrate({this:?}, {duration})");

        Ok(false)
    }

    async fn repaint(
        _jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::repaint({this:?}, {x}, {y}, {width}, {height})");

        let platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

        Ok(())
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

        if !current_displayable.is_null() {
            let result: JvmResult<()> = jvm
                .invoke_virtual(&current_displayable, "handleKeyEvent", "(II)V", (event_type, code))
                .await;

            if let Err(x) = result {
                Self::handle_exception(jvm, x).await?;
            }
        }

        Ok(())
    }

    async fn handle_paint_event(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::handlePaintEvent({:?})", &this);

        let current_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        if !current_displayable.is_null() {
            let screen_graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;

            // TODO draw title and bottom soft bar if not fullscreen

            let result: JvmResult<()> = jvm
                .invoke_virtual(
                    &current_displayable,
                    "handlePaintEvent",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    (screen_graphics.clone(),),
                )
                .await;
            let _: () = jvm.invoke_virtual(&screen_graphics, "reset", "()V", ()).await?;

            if let Err(x) = result {
                Self::handle_exception(jvm, x).await?;
            }

            // HACK: disable paint for clet apps, as they handle paint by themselves
            let disable_paint: bool = jvm.get_field(&this, "paintDisabled", "Z").await?;
            if !disable_paint {
                let screen_image: ClassInstanceRef<Image> = jvm.get_field(&this, "screenImage", "Ljavax/microedition/lcdui/Image;").await?;
                let image = Image::image(jvm, &screen_image).await?;

                let platform = context.system().platform();
                let screen = platform.screen();

                screen.paint(&*image);
            }
            jvm.collect_garbage()?;
        }

        Ok(())
    }

    async fn handle_notify_event(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: i32,
        param1: i32,
        param2: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Display::handleNotifyEvent({this:?}, {}, {param1}, {param2})",
            r#type,
        );

        let current_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        if !current_displayable.is_null() {
            let result: JvmResult<()> = jvm
                .invoke_virtual(&current_displayable, "handleNotifyEvent", "(III)V", (r#type, param1, param2))
                .await;

            if let Err(x) = result {
                Self::handle_exception(jvm, x).await?;
            }
        }

        Ok(())
    }

    pub async fn screen_graphics(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        jvm.get_field(this, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;").await
    }

    async fn handle_exception(jvm: &Jvm, err: JavaError) -> JvmResult<()> {
        if let JavaError::JavaException(x) = err {
            if jvm.is_instance(&*x, "java/lang/Error") {
                return Err(JavaError::JavaException(x));
            }

            let string_writer = jvm.new_class("java/io/StringWriter", "()V", ()).await?;
            let print_writer = jvm
                .new_class("java/io/PrintWriter", "(Ljava/io/Writer;)V", (string_writer.clone(),))
                .await?;

            let _: () = jvm
                .invoke_virtual(&x, "printStackTrace", "(Ljava/io/PrintWriter;)V", (print_writer,))
                .await?;

            let trace = jvm.invoke_virtual(&string_writer, "toString", "()Ljava/lang/String;", []).await?;
            let trace = JavaLangString::to_rust_string(jvm, &trace).await?;

            tracing::warn!("Exception while event handling: {}", trace);

            Ok(())
        } else {
            Err(err)
        }
    }

    async fn set_fullscreen(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, fullscreen: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::setFullscreen({this:?}, {fullscreen})");

        jvm.put_field(&mut this, "isInFullScreenMode", "Z", fullscreen).await?;

        let platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

        Ok(())
    }

    async fn disable_paint(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::disablePaint({this:?})");

        jvm.put_field(&mut this, "paintDisabled", "Z", true).await?;

        Ok(())
    }
}
