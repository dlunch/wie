use alloc::{string::String as RustString, vec, vec::Vec};

use jvm::{ClassInstanceRef, JavaError, JavaValue, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::{Runnable, String as JavaString};

use wie_backend::Event;
use wie_jvm_support::{JvmSupport, WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::{
    lcdui::{Alert, Displayable, Font, Graphics, Image, Ticker},
    midlet::MIDlet,
};

const FONT_HEIGHT: i32 = 12;
const TITLE_HORIZONTAL_PADDING: i32 = 4;
const TITLE_VERTICAL_PADDING: i32 = 2;
const TICKER_HEIGHT: i32 = 16;
const TICKER_INTERVAL_MS: u64 = 100;
const SOFTKEY_HEIGHT: i32 = 18;

const TITLE_BACKGROUND: i32 = 0x263746;
const WHITE: i32 = 0xffffff;

const LEFT_TOP: i32 = 4 | 16;

struct ChromeLayout {
    content_width: i32,
    content_height: i32,
    content_x: i32,
    content_y: i32,
    title_height: i32,
    title_lines: Vec<RustString>,
    ticker_y: i32,
    ticker_height: i32,
    softkey_y: i32,
    softkey_height: i32,
}

// class javax.microedition.lcdui.Display
pub struct Display;

impl Display {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Display",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    Self::set_current,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Alert;Ljavax/microedition/lcdui/Displayable;)V",
                    Self::set_current_alert,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getCurrent",
                    "()Ljavax/microedition/lcdui/Displayable;",
                    Self::get_current,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::empty()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::empty()),
                JavaMethodProto::new("callSerially", "(Ljava/lang/Runnable;)V", Self::call_serially, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("vibrate", "(I)Z", Self::vibrate, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                    Self::get_display,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                // wie private methods...
                JavaMethodProto::new(
                    "transition",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    Self::transition,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("alertChanged", "()V", Self::alert_changed, MethodAccessFlags::empty()),
                JavaMethodProto::new("tickerChanged", "()V", Self::ticker_changed, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "getVisibleTicker",
                    "()Ljavax/microedition/lcdui/Ticker;",
                    Self::visible_ticker,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "getScreenGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    Self::screen_graphics,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "getContentHeight",
                    "(Ljavax/microedition/lcdui/Displayable;II)I",
                    Self::get_content_height,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, MethodAccessFlags::empty()),
                JavaMethodProto::new("handlePaintEvent", "()V", Self::handle_paint_event, MethodAccessFlags::empty()),
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::empty()),
                JavaMethodProto::new("handleNotifyEvent", "(III)V", Self::handle_notify_event, MethodAccessFlags::empty()),
                JavaMethodProto::new("handleAlertTimeout", "(I)V", Self::handle_alert_timeout, MethodAccessFlags::empty()),
                JavaMethodProto::new("handleTickerTick", "(I)V", Self::handle_ticker_tick, MethodAccessFlags::empty()),
                JavaMethodProto::new("setFullscreen", "(Z)V", Self::set_fullscreen, MethodAccessFlags::empty()),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint, MethodAccessFlags::empty()),
                JavaMethodProto::new("disablePaint", "()V", Self::disable_paint, MethodAccessFlags::empty()),
            ],
            fields: vec![
                JavaFieldProto::new("isInFullScreenMode", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("currentDisplayable", "Ljavax/microedition/lcdui/Displayable;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("screenImage", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("screenGraphics", "Ljavax/microedition/lcdui/Graphics;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("width", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("paintDisabled", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("repaintPending", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("alertGeneration", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tickerGeneration", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::<init>({this:?})");

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
            .invoke_virtual(
                &screen_image,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;

        jvm.put_field(&mut this, "screenImage", "Ljavax/microedition/lcdui/Image;", screen_image)
            .await?;
        jvm.put_field(&mut this, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;", screen_graphics)
            .await?;

        Ok(())
    }

    async fn get_content_height(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        displayable: ClassInstanceRef<Displayable>,
        width: i32,
        height: i32,
    ) -> JvmResult<i32> {
        let layout = Self::chrome_layout(jvm, context, &displayable, width, height).await?;
        Ok(layout.content_height)
    }

    async fn chrome_layout(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        displayable: &ClassInstanceRef<Displayable>,
        width: i32,
        height: i32,
    ) -> JvmResult<ChromeLayout> {
        let width = width.max(0);
        let height = height.max(0);
        let fullscreen: bool = jvm
            .invoke_virtual(displayable, "javax/microedition/lcdui/Displayable", "isFullScreen", "()Z", ())
            .await?;
        if fullscreen {
            return Ok(ChromeLayout {
                content_x: 0,
                content_y: 0,
                content_width: width,
                content_height: height,
                title_height: 0,
                title_lines: Vec::new(),
                ticker_y: 0,
                ticker_height: 0,
                softkey_y: height,
                softkey_height: 0,
            });
        }

        let command_count: i32 = jvm
            .invoke_virtual(displayable, "javax/microedition/lcdui/Displayable", "getCommandCount", "()I", ())
            .await?;
        let ticker: ClassInstanceRef<Ticker> = jvm
            .invoke_virtual(
                displayable,
                "javax/microedition/lcdui/Displayable",
                "getTicker",
                "()Ljavax/microedition/lcdui/Ticker;",
                (),
            )
            .await?;
        let title: ClassInstanceRef<JavaString> = jvm
            .invoke_virtual(
                displayable,
                "javax/microedition/lcdui/Displayable",
                "getTitle",
                "()Ljava/lang/String;",
                (),
            )
            .await?;
        let title_lines = if title.is_null() {
            Vec::new()
        } else {
            let title = JavaLangString::to_rust_string(jvm, &title).await?;
            Font::wrap(
                context.system().platform().font(),
                &title,
                Some((width - TITLE_HORIZONTAL_PADDING * 2).max(1)),
            )
        };

        let softkey_height = if command_count > 0 { SOFTKEY_HEIGHT.min(height) } else { 0 };
        let ticker_height = if ticker.is_null() {
            0
        } else {
            TICKER_HEIGHT.min((height - softkey_height).max(0))
        };
        let desired_title_height = if title_lines.is_empty() {
            0
        } else {
            title_lines.len() as i32 * FONT_HEIGHT + TITLE_VERTICAL_PADDING * 2
        };
        let title_height = desired_title_height.min((height - softkey_height - ticker_height).max(0));
        let content_y = title_height + ticker_height;
        let content_height = (height - content_y - softkey_height).max(0);

        Ok(ChromeLayout {
            content_x: 0,
            content_y,
            content_width: width,
            content_height,
            title_height,
            title_lines,
            ticker_y: title_height,
            ticker_height,
            softkey_y: height - softkey_height,
            softkey_height,
        })
    }

    async fn get_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Display::getWidth({this:?})");

        let width = jvm.get_field(&this, "width", "I").await?;

        Ok(width)
    }

    async fn get_height(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Display::getHeight({this:?})");

        let height = jvm.get_field(&this, "height", "I").await?;

        Ok(height)
    }

    async fn call_serially(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event: ClassInstanceRef<Runnable>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::callSerially({this:?}, {event:?})");

        let event_queue = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(&event_queue, "net/wie/EventQueue", "callSerially", "(Ljava/lang/Runnable;)V", (event,))
            .await?;

        Ok(())
    }

    async fn set_current(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        displayable: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::setCurrent({this:?}, {displayable:?})");

        if displayable.is_null() {
            return Ok(());
        }

        if jvm.is_instance(&**displayable, "javax/microedition/lcdui/Alert") {
            let current: ClassInstanceRef<Displayable> = jvm
                .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
                .await?;
            if !current.is_null() && jvm.is_instance(&**current, "javax/microedition/lcdui/Alert") {
                return Err(jvm
                    .exception("java/lang/IllegalArgumentException", "An Alert cannot follow an Alert")
                    .await);
            }
            let alert: ClassInstanceRef<Alert> = JavaValue::from(displayable.clone()).into();
            jvm.invoke_virtual::<_, ()>(
                &alert,
                "javax/microedition/lcdui/Alert",
                "setNextDisplayable",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (current,),
            )
            .await?;
        }

        jvm.invoke_virtual(
            &this,
            "javax/microedition/lcdui/Display",
            "transition",
            "(Ljavax/microedition/lcdui/Displayable;)V",
            (displayable,),
        )
        .await
    }

    async fn set_current_alert(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        alert: ClassInstanceRef<Alert>,
        next_displayable: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::setCurrent({this:?}, {alert:?}, {next_displayable:?})");

        if alert.is_null() || next_displayable.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "Alert transition argument is null").await);
        }
        if jvm.is_instance(&**next_displayable, "javax/microedition/lcdui/Alert") {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "An Alert cannot follow an Alert")
                .await);
        }

        jvm.invoke_virtual::<_, ()>(
            &alert,
            "javax/microedition/lcdui/Alert",
            "setNextDisplayable",
            "(Ljavax/microedition/lcdui/Displayable;)V",
            (next_displayable,),
        )
        .await?;
        let displayable: ClassInstanceRef<Displayable> = JavaValue::from(alert).into();
        jvm.invoke_virtual(
            &this,
            "javax/microedition/lcdui/Display",
            "transition",
            "(Ljavax/microedition/lcdui/Displayable;)V",
            (displayable,),
        )
        .await
    }

    async fn transition(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        displayable: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        let alert_generation: i32 = jvm.get_field(&this, "alertGeneration", "I").await?;
        let alert_generation = alert_generation.wrapping_add(1);
        jvm.put_field(&mut this, "alertGeneration", "I", alert_generation).await?;
        let ticker_generation: i32 = jvm.get_field(&this, "tickerGeneration", "I").await?;
        let ticker_generation = ticker_generation.wrapping_add(1);
        jvm.put_field(&mut this, "tickerGeneration", "I", ticker_generation).await?;

        let old_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        let same_displayable = !old_displayable.is_null() && !displayable.is_null() && old_displayable.identity() == displayable.identity();
        if !old_displayable.is_null() && !same_displayable {
            let _: () = jvm
                .invoke_virtual(
                    &old_displayable,
                    "javax/microedition/lcdui/Displayable",
                    "setDisplay",
                    "(Ljavax/microedition/lcdui/Display;)V",
                    (None,),
                )
                .await?;
        }

        jvm.put_field(
            &mut this,
            "currentDisplayable",
            "Ljavax/microedition/lcdui/Displayable;",
            displayable.clone(),
        )
        .await?;

        if displayable.is_null() {
            jvm.put_field(&mut this, "isInFullScreenMode", "Z", false).await?;
            let width: i32 = jvm.get_field(&this, "width", "I").await?;
            let height: i32 = jvm.get_field(&this, "height", "I").await?;
            return jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, width, height))
                .await;
        }

        if !same_displayable {
            let _: () = jvm
                .invoke_virtual(
                    &displayable,
                    "javax/microedition/lcdui/Displayable",
                    "setDisplay",
                    "(Ljavax/microedition/lcdui/Display;)V",
                    (this.clone(),),
                )
                .await?;
        }

        let fullscreen_mode: bool = jvm
            .invoke_virtual(&displayable, "javax/microedition/lcdui/Displayable", "isFullScreen", "()Z", ())
            .await?;
        jvm.put_field(&mut this, "isInFullScreenMode", "Z", fullscreen_mode).await?;

        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;
        let notification_result: JvmResult<()> = jvm
            .invoke_virtual(&displayable, "javax/microedition/lcdui/Displayable", "notifySizeChanged", "()V", ())
            .await;
        let timer_result = Self::schedule_alert_timeout(jvm, context, this.clone(), alert_generation).await;
        let ticker_result = Self::schedule_ticker_tick(jvm, context, this.clone(), ticker_generation).await;
        let repaint_result: JvmResult<()> = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, width, height))
            .await;
        notification_result?;
        timer_result?;
        ticker_result?;
        repaint_result?;

        Ok(())
    }

    async fn schedule_alert_timeout(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, generation: i32) -> JvmResult<()> {
        let current_generation: i32 = jvm.get_field(&this, "alertGeneration", "I").await?;
        if current_generation != generation {
            return Ok(());
        }

        let current: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
        if current.is_null() || !jvm.is_instance(&**current, "javax/microedition/lcdui/Alert") {
            return Ok(());
        }
        let attached_display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(
                &current,
                "javax/microedition/lcdui/Displayable",
                "getDisplay",
                "()Ljavax/microedition/lcdui/Display;",
                (),
            )
            .await?;
        if attached_display.is_null() || attached_display.identity() != this.identity() {
            return Ok(());
        }

        let timeout: i32 = jvm
            .invoke_virtual(&current, "javax/microedition/lcdui/Alert", "getTimeout", "()I", ())
            .await?;
        if timeout <= 0 {
            return Ok(());
        }

        let due = context.system().platform().now() + timeout as u64;
        let timer_jvm = jvm.clone();
        context.system().event_queue().push(Event::timer(due, move || {
            let jvm = timer_jvm;
            async move {
                let result: JvmResult<()> = async {
                    let midlet: ClassInstanceRef<MIDlet> = jvm
                        .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
                        .await?;
                    if midlet.is_null() {
                        return Ok(());
                    }

                    let display: ClassInstanceRef<Display> = jvm
                        .invoke_static(
                            "javax/microedition/lcdui/Display",
                            "getDisplay",
                            "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                            (midlet,),
                        )
                        .await?;
                    jvm.invoke_virtual(&display, "javax/microedition/lcdui/Display", "handleAlertTimeout", "(I)V", (generation,))
                        .await
                }
                .await;

                match result {
                    Ok(()) => Ok(()),
                    Err(error) => Err(JvmSupport::to_wie_err(&jvm, error).await),
                }
            }
        }));

        Ok(())
    }

    async fn alert_changed(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let generation: i32 = jvm.get_field(&this, "alertGeneration", "I").await?;
        let generation = generation.wrapping_add(1);
        jvm.put_field(&mut this, "alertGeneration", "I", generation).await?;
        Self::schedule_alert_timeout(jvm, context, this, generation).await
    }

    async fn visible_ticker(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Ticker>> {
        let current: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
        if current.is_null() {
            return Ok(None.into());
        }
        let attached_display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(
                &current,
                "javax/microedition/lcdui/Displayable",
                "getDisplay",
                "()Ljavax/microedition/lcdui/Display;",
                (),
            )
            .await?;
        if attached_display.is_null() || attached_display.identity() != this.identity() {
            return Ok(None.into());
        }

        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;
        let layout = Self::chrome_layout(jvm, context, &current, width, height).await?;
        if layout.ticker_height == 0 || layout.content_width == 0 {
            return Ok(None.into());
        }
        jvm.invoke_virtual(
            &current,
            "javax/microedition/lcdui/Displayable",
            "getTicker",
            "()Ljavax/microedition/lcdui/Ticker;",
            (),
        )
        .await
    }

    async fn ticker_changed(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let generation: i32 = jvm.get_field(&this, "tickerGeneration", "I").await?;
        let generation = generation.wrapping_add(1);
        jvm.put_field(&mut this, "tickerGeneration", "I", generation).await?;
        Self::schedule_ticker_tick(jvm, context, this, generation).await
    }

    async fn schedule_ticker_tick(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, generation: i32) -> JvmResult<()> {
        if jvm.get_field::<i32>(&this, "tickerGeneration", "I").await? != generation {
            return Ok(());
        }
        let ticker: ClassInstanceRef<Ticker> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Display",
                "getVisibleTicker",
                "()Ljavax/microedition/lcdui/Ticker;",
                (),
            )
            .await?;
        if ticker.is_null() {
            return Ok(());
        }
        let text: ClassInstanceRef<JavaString> = jvm
            .invoke_virtual(&ticker, "javax/microedition/lcdui/Ticker", "getString", "()Ljava/lang/String;", ())
            .await?;
        if JavaLangString::to_rust_string(jvm, &text).await?.is_empty() {
            return Ok(());
        }

        let due = context.system().platform().now() + TICKER_INTERVAL_MS;
        let timer_jvm = jvm.clone();
        context.system().event_queue().push(Event::timer(due, move || {
            let jvm = timer_jvm;
            async move {
                let result: JvmResult<()> = async {
                    let midlet: ClassInstanceRef<MIDlet> = jvm
                        .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
                        .await?;
                    if midlet.is_null() {
                        return Ok(());
                    }
                    let display: ClassInstanceRef<Display> = jvm
                        .invoke_static(
                            "javax/microedition/lcdui/Display",
                            "getDisplay",
                            "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                            (midlet,),
                        )
                        .await?;
                    jvm.invoke_virtual(&display, "javax/microedition/lcdui/Display", "handleTickerTick", "(I)V", (generation,))
                        .await
                }
                .await;
                match result {
                    Ok(()) => Ok(()),
                    Err(error) => Err(JvmSupport::to_wie_err(&jvm, error).await),
                }
            }
        }));

        Ok(())
    }

    async fn handle_ticker_tick(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, generation: i32) -> JvmResult<()> {
        if jvm.get_field::<i32>(&this, "tickerGeneration", "I").await? != generation {
            return Ok(());
        }
        let ticker: ClassInstanceRef<Ticker> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Display",
                "getVisibleTicker",
                "()Ljavax/microedition/lcdui/Ticker;",
                (),
            )
            .await?;
        if ticker.is_null() {
            return Ok(());
        }
        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let advanced: bool = jvm
            .invoke_virtual(&ticker, "javax/microedition/lcdui/Ticker", "advance", "(I)Z", (width,))
            .await?;
        if !advanced {
            return Ok(());
        }
        let _: () = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, -1, -1))
            .await?;
        Self::schedule_ticker_tick(jvm, context, this, generation).await
    }

    async fn get_current(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Displayable>> {
        tracing::debug!("javax.microedition.lcdui.Display::getCurrent({this:?})");

        let current_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        Ok(current_displayable)
    }

    async fn get_display(jvm: &Jvm, _context: &mut WieJvmContext, midlet: ClassInstanceRef<MIDlet>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("javax.microedition.lcdui.Display::getDisplay({midlet:?})");

        let display = MIDlet::display(jvm, &midlet).await?;

        Ok(display)
    }

    async fn vibrate(_jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, duration: i32) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.Display::vibrate({this:?}, {duration})");

        context.system().platform().vibrate(duration.max(0) as u64, 100);

        Ok(true)
    }

    async fn repaint(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::repaint({this:?}, {x}, {y}, {width}, {height})");

        jvm.put_field(&mut this, "repaintPending", "Z", true).await?;
        let platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

        Ok(())
    }

    async fn handle_key_event(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::handleKeyEvent({this:?}, {event_type:?}, {code})");

        let current_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;

        if !current_displayable.is_null() {
            let result: JvmResult<()> = jvm
                .invoke_virtual(
                    &current_displayable,
                    "javax/microedition/lcdui/Displayable",
                    "routeKeyEvent",
                    "(II)V",
                    (event_type, code),
                )
                .await;

            if let Err(x) = result {
                Self::handle_exception(jvm, x).await?;
            }
        }

        Ok(())
    }

    async fn handle_alert_timeout(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, generation: i32) -> JvmResult<()> {
        let current_generation: i32 = jvm.get_field(&this, "alertGeneration", "I").await?;
        if current_generation != generation {
            return Ok(());
        }

        let current: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
        if current.is_null() || !jvm.is_instance(&**current, "javax/microedition/lcdui/Alert") {
            return Ok(());
        }
        let attached_display: ClassInstanceRef<Display> = jvm
            .invoke_virtual(
                &current,
                "javax/microedition/lcdui/Displayable",
                "getDisplay",
                "()Ljavax/microedition/lcdui/Display;",
                (),
            )
            .await?;
        if attached_display.is_null() || attached_display.identity() != this.identity() {
            return Ok(());
        }

        let result: JvmResult<()> = jvm
            .invoke_virtual(&current, "javax/microedition/lcdui/Displayable", "dispatchCommandAt", "(I)V", (0,))
            .await;
        if let Err(error) = result {
            Self::handle_exception(jvm, error).await?;
        }
        Ok(())
    }

    async fn service_repaints(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        if jvm.get_field::<bool>(&this, "repaintPending", "Z").await? {
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                .await?;
        }
        Ok(())
    }

    async fn handle_paint_event(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::handlePaintEvent({this:?})");

        // Repaints requested during painting belong to the next cycle.
        jvm.put_field(&mut this, "repaintPending", "Z", false).await?;
        let current_displayable: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
        let screen_graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;

        if current_displayable.is_null() {
            let _: () = jvm
                .invoke_virtual(&screen_graphics, "javax/microedition/lcdui/Graphics", "reset", "()V", ())
                .await?;
            let _: () = jvm
                .invoke_virtual(&screen_graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (WHITE,))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &screen_graphics,
                    "javax/microedition/lcdui/Graphics",
                    "fillRect",
                    "(IIII)V",
                    (0, 0, width, height),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&screen_graphics, "javax/microedition/lcdui/Graphics", "reset", "()V", ())
                .await?;
        } else {
            let notification_result: JvmResult<()> = jvm
                .invoke_virtual(
                    &current_displayable,
                    "javax/microedition/lcdui/Displayable",
                    "notifySizeChanged",
                    "()V",
                    (),
                )
                .await;
            if let Err(error) = notification_result {
                Self::handle_exception(jvm, error).await?;
            }

            let layout = Self::chrome_layout(jvm, context, &current_displayable, width, height).await?;
            let _: () = jvm
                .invoke_virtual(&screen_graphics, "javax/microedition/lcdui/Graphics", "reset", "()V", ())
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &screen_graphics,
                    "javax/microedition/lcdui/Graphics",
                    "translate",
                    "(II)V",
                    (layout.content_x, layout.content_y),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &screen_graphics,
                    "javax/microedition/lcdui/Graphics",
                    "setClip",
                    "(IIII)V",
                    (0, 0, layout.content_width, layout.content_height),
                )
                .await?;

            let result: JvmResult<()> = jvm
                .invoke_virtual(
                    &current_displayable,
                    "javax/microedition/lcdui/Displayable",
                    "handlePaintEvent",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    (screen_graphics.clone(),),
                )
                .await;
            let _: () = jvm
                .invoke_virtual(&screen_graphics, "javax/microedition/lcdui/Graphics", "reset", "()V", ())
                .await?;

            if let Err(x) = result {
                Self::handle_exception(jvm, x).await?;
            }

            let chrome_result = Self::paint_chrome(jvm, current_displayable, screen_graphics.clone(), &layout).await;
            let _: () = jvm
                .invoke_virtual(&screen_graphics, "javax/microedition/lcdui/Graphics", "reset", "()V", ())
                .await?;
            if let Err(error) = chrome_result {
                Self::handle_exception(jvm, error).await?;
            }
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

        Ok(())
    }

    async fn paint_chrome(
        jvm: &Jvm,
        displayable: ClassInstanceRef<Displayable>,
        graphics: ClassInstanceRef<Graphics>,
        layout: &ChromeLayout,
    ) -> JvmResult<()> {
        if layout.title_height > 0 {
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "setClip",
                    "(IIII)V",
                    (0, 0, layout.content_width, layout.title_height),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (TITLE_BACKGROUND,))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "fillRect",
                    "(IIII)V",
                    (0, 0, layout.content_width, layout.title_height),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (WHITE,))
                .await?;
            for (index, line) in layout.title_lines.iter().enumerate() {
                let line = JavaLangString::from_rust_string(jvm, line).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &graphics,
                        "javax/microedition/lcdui/Graphics",
                        "drawString",
                        "(Ljava/lang/String;III)V",
                        (
                            line,
                            TITLE_HORIZONTAL_PADDING,
                            TITLE_VERTICAL_PADDING + index as i32 * FONT_HEIGHT,
                            LEFT_TOP,
                        ),
                    )
                    .await?;
            }
        }

        if layout.ticker_height > 0 {
            let ticker: ClassInstanceRef<Ticker> = jvm
                .invoke_virtual(
                    &displayable,
                    "javax/microedition/lcdui/Displayable",
                    "getTicker",
                    "()Ljavax/microedition/lcdui/Ticker;",
                    (),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &ticker,
                    "javax/microedition/lcdui/Ticker",
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;III)V",
                    (graphics.clone(), layout.content_width, layout.ticker_y, layout.ticker_height),
                )
                .await?;
        }

        if layout.softkey_height > 0 {
            let _: () = jvm
                .invoke_virtual(
                    &displayable,
                    "javax/microedition/lcdui/Displayable",
                    "paintCommands",
                    "(Ljavax/microedition/lcdui/Graphics;III)V",
                    (graphics, layout.content_width, layout.softkey_y, layout.softkey_height),
                )
                .await?;
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
                .invoke_virtual(
                    &current_displayable,
                    "javax/microedition/lcdui/Displayable",
                    "handleNotifyEvent",
                    "(III)V",
                    (r#type, param1, param2),
                )
                .await;

            if let Err(x) = result {
                Self::handle_exception(jvm, x).await?;
            }
        }

        Ok(())
    }

    async fn screen_graphics(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        jvm.get_field(&this, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;").await
    }

    async fn handle_exception(jvm: &Jvm, err: JavaError) -> JvmResult<()> {
        let JavaError::JavaException(x) = err;

        if jvm.is_instance(&*x, "java/lang/Error") {
            return Err(JavaError::JavaException(x));
        }

        let string_writer = jvm.new_class("java/io/StringWriter", "()V", ()).await?;
        let print_writer = jvm
            .new_class("java/io/PrintWriter", "(Ljava/io/Writer;)V", (string_writer.clone(),))
            .await?;

        let _: () = jvm
            .invoke_virtual(&x, "java/lang/Throwable", "printStackTrace", "(Ljava/io/PrintWriter;)V", (print_writer,))
            .await?;

        let trace = jvm
            .invoke_virtual(&string_writer, "java/io/StringWriter", "toString", "()Ljava/lang/String;", [])
            .await?;
        let trace = JavaLangString::to_rust_string(jvm, &trace).await?;

        tracing::warn!("Exception while event handling: {trace}");

        Ok(())
    }

    async fn set_fullscreen(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, fullscreen: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::setFullscreen({this:?}, {fullscreen})");

        jvm.put_field(&mut this, "isInFullScreenMode", "Z", fullscreen).await?;
        let _: () = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Display", "tickerChanged", "()V", ())
            .await?;
        let current: ClassInstanceRef<Displayable> = jvm
            .get_field(&this, "currentDisplayable", "Ljavax/microedition/lcdui/Displayable;")
            .await?;
        if !current.is_null() {
            let _: () = jvm
                .invoke_virtual(&current, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
                .await?;
        } else {
            let platform = context.system().platform();
            platform.screen().request_redraw().unwrap();
        }

        Ok(())
    }

    async fn disable_paint(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::disablePaint({this:?})");

        jvm.put_field(&mut this, "paintDisabled", "Z", true).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstanceRef, JavaValue, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::{
        classes::{
            javax::microedition::lcdui::{Command, CommandListener, Display, Displayable, Graphics, Image},
            net::wie::{KeyboardEventType, MIDPKeyCode},
        },
        get_protos,
    };

    struct ViewportScreen;
    struct RecordingCommandListener;

    impl ViewportScreen {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestViewportScreen",
                parent_class: Some("javax/microedition/lcdui/Screen"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("sizeChanged", "(II)V", Self::size_changed, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new(
                        "handlePaintEvent",
                        "(Ljavax/microedition/lcdui/Graphics;)V",
                        Self::handle_paint_event,
                        MethodAccessFlags::empty(),
                    ),
                    JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::empty()),
                ],
                fields: vec![
                    JavaFieldProto::new("callbackCount", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("callbackDepth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("maximumCallbackDepth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastHeight", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("mutateOnFirstCallback", "Z", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("translateX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("translateY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("clipHeight", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("keyCount", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastKeyType", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastKeyCode", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await
        }

        async fn size_changed(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
            let callback_count: i32 = jvm.get_field(&this, "callbackCount", "I").await?;
            let callback_depth: i32 = jvm.get_field(&this, "callbackDepth", "I").await?;
            let callback_count = callback_count + 1;
            let callback_depth = callback_depth + 1;

            jvm.put_field(&mut this, "callbackCount", "I", callback_count).await?;
            jvm.put_field(&mut this, "callbackDepth", "I", callback_depth).await?;
            jvm.put_field(&mut this, "lastWidth", "I", width).await?;
            jvm.put_field(&mut this, "lastHeight", "I", height).await?;

            let maximum_depth: i32 = jvm.get_field(&this, "maximumCallbackDepth", "I").await?;
            if callback_depth > maximum_depth {
                jvm.put_field(&mut this, "maximumCallbackDepth", "I", callback_depth).await?;
            }

            let mutate: bool = jvm.get_field(&this, "mutateOnFirstCallback", "Z").await?;
            if mutate && callback_count == 1 {
                let title = JavaLangString::from_rust_string(jvm, "Changed\ninside callback").await?;
                let _: () = jvm
                    .invoke_virtual(
                        &this,
                        "javax/microedition/lcdui/Displayable",
                        "setTitle",
                        "(Ljava/lang/String;)V",
                        (title,),
                    )
                    .await?;

                let label = JavaLangString::from_rust_string(jvm, "Done").await?;
                let command: ClassInstanceRef<Command> = jvm
                    .new_class("javax/microedition/lcdui/Command", "(Ljava/lang/String;II)V", (label, 4, 0))
                    .await?
                    .into();
                let _: () = jvm
                    .invoke_virtual(
                        &this,
                        "javax/microedition/lcdui/Displayable",
                        "addCommand",
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command,),
                    )
                    .await?;
            }

            jvm.put_field(&mut this, "callbackDepth", "I", callback_depth - 1).await?;
            Ok(())
        }

        async fn handle_paint_event(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            graphics: ClassInstanceRef<Graphics>,
        ) -> JvmResult<()> {
            let _: () = jvm
                .invoke_special(
                    &this,
                    "javax/microedition/lcdui/Screen",
                    "handlePaintEvent",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    (graphics.clone(),),
                )
                .await?;

            for (field, method) in [
                ("translateX", "getTranslateX"),
                ("translateY", "getTranslateY"),
                ("clipX", "getClipX"),
                ("clipY", "getClipY"),
                ("clipWidth", "getClipWidth"),
                ("clipHeight", "getClipHeight"),
            ] {
                let value: i32 = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", method, "()I", ())
                    .await?;
                jvm.put_field(&mut this, field, "I", value).await?;
            }

            let width: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
                .await?;
            let height: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                .await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xcc1133,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (-4, -4, 8, 8))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x1188cc,))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "fillRect",
                    "(IIII)V",
                    (width - 2, height - 2, 6, 6),
                )
                .await?;

            Ok(())
        }

        async fn handle_key_event(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            event_type: i32,
            code: i32,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "keyCount", "I").await?;
            jvm.put_field(&mut this, "keyCount", "I", count + 1).await?;
            jvm.put_field(&mut this, "lastKeyType", "I", event_type).await?;
            jvm.put_field(&mut this, "lastKeyCode", "I", code).await
        }
    }

    impl RecordingCommandListener {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestRecordingCommandListener",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["javax/microedition/lcdui/CommandListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new(
                        "commandAction",
                        "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V",
                        Self::command_action,
                        MethodAccessFlags::PUBLIC,
                    ),
                ],
                fields: vec![
                    JavaFieldProto::new("count", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastCommand", "Ljavax/microedition/lcdui/Command;", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastDisplayable", "Ljavax/microedition/lcdui/Displayable;", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn command_action(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            command: ClassInstanceRef<Command>,
            displayable: ClassInstanceRef<Displayable>,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "count", "I").await?;
            jvm.put_field(&mut this, "count", "I", count + 1).await?;
            jvm.put_field(&mut this, "lastCommand", "Ljavax/microedition/lcdui/Command;", command)
                .await?;
            jvm.put_field(&mut this, "lastDisplayable", "Ljavax/microedition/lcdui/Displayable;", displayable)
                .await?;

            Ok(())
        }
    }

    fn test_protos() -> Box<[Box<[WieJavaClassProto]>]> {
        Box::new([
            get_protos().into(),
            [ViewportScreen::as_proto(), RecordingCommandListener::as_proto()].into(),
        ])
    }

    async fn set_title(jvm: &Jvm, screen: &ClassInstanceRef<ViewportScreen>, title: &str) -> JvmResult<()> {
        let title = JavaLangString::from_rust_string(jvm, title).await?;
        jvm.invoke_virtual(
            screen,
            "javax/microedition/lcdui/Displayable",
            "setTitle",
            "(Ljava/lang/String;)V",
            (title,),
        )
        .await
    }

    async fn viewport_height(jvm: &Jvm, screen: &ClassInstanceRef<ViewportScreen>) -> JvmResult<i32> {
        jvm.invoke_virtual(screen, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await
    }

    async fn make_command(jvm: &Jvm, label: &str, command_type: i32, priority: i32) -> JvmResult<ClassInstanceRef<Command>> {
        let label = JavaLangString::from_rust_string(jvm, label).await?;
        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Command",
                "(Ljava/lang/String;II)V",
                (label, command_type, priority),
            )
            .await?
            .into())
    }

    async fn send_key(jvm: &Jvm, display: &ClassInstanceRef<Display>, event_type: KeyboardEventType, key: MIDPKeyCode) -> JvmResult<()> {
        jvm.invoke_virtual(
            display,
            "javax/microedition/lcdui/Display",
            "handleKeyEvent",
            "(II)V",
            (event_type as i32, key as i32),
        )
        .await
    }

    #[test]
    fn hidden_and_attached_viewports_follow_live_decorations() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let screen: ClassInstanceRef<ViewportScreen> = jvm.new_class("javax/microedition/lcdui/TestViewportScreen", "()V", ()).await?.into();

            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&screen, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
                    .await?,
                320
            );
            let undecorated_height = viewport_height(&jvm, &screen).await?;
            assert_eq!(undecorated_height, 240);

            set_title(&jvm, &screen, "One line").await?;
            let one_line_height = viewport_height(&jvm, &screen).await?;
            assert!(one_line_height < undecorated_height);

            set_title(&jvm, &screen, "First line\nSecond line").await?;
            let explicit_newline_height = viewport_height(&jvm, &screen).await?;
            assert!(explicit_newline_height < one_line_height);

            set_title(
                &jvm,
                &screen,
                "A deliberately long title whose words must wrap because the available display width is finite and deterministic",
            )
            .await?;
            let wrapped_height = viewport_height(&jvm, &screen).await?;
            assert!(wrapped_height < one_line_height);

            set_title(&jvm, &screen, "Decorated").await?;
            let title_height = viewport_height(&jvm, &screen).await?;
            let command = make_command(&jvm, "Open", 4, 0).await?;
            let _: () = jvm
                .invoke_virtual(
                    &screen,
                    "javax/microedition/lcdui/Displayable",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (command.clone(),),
                )
                .await?;
            let fully_decorated_height = viewport_height(&jvm, &screen).await?;
            assert!(fully_decorated_height < title_height);
            assert_eq!(jvm.get_field::<i32>(&screen, "callbackCount", "I").await?, 0);

            let display: ClassInstanceRef<Display> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    (screen.clone(),),
                )
                .await?;
            assert_eq!(jvm.get_field::<i32>(&screen, "callbackCount", "I").await?, 1);
            assert!(
                jvm.invoke_virtual::<_, bool>(&screen, "javax/microedition/lcdui/Displayable", "isShown", "()Z", ())
                    .await?
            );
            assert_eq!(viewport_height(&jvm, &screen).await?, fully_decorated_height);

            let _: () = jvm
                .invoke_virtual(
                    &screen,
                    "javax/microedition/lcdui/Displayable",
                    "removeCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (command.clone(),),
                )
                .await?;
            assert_eq!(jvm.get_field::<i32>(&screen, "callbackCount", "I").await?, 2);
            assert_eq!(viewport_height(&jvm, &screen).await?, title_height);
            let _: () = jvm
                .invoke_virtual(
                    &screen,
                    "javax/microedition/lcdui/Displayable",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (command,),
                )
                .await?;
            assert_eq!(jvm.get_field::<i32>(&screen, "callbackCount", "I").await?, 3);
            assert_eq!(viewport_height(&jvm, &screen).await?, fully_decorated_height);

            let replacement: ClassInstanceRef<ViewportScreen> = jvm.new_class("javax/microedition/lcdui/TestViewportScreen", "()V", ()).await?.into();
            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    (replacement,),
                )
                .await?;
            assert!(
                !jvm.invoke_virtual::<_, bool>(&screen, "javax/microedition/lcdui/Displayable", "isShown", "()Z", ())
                    .await?
            );

            let _: () = jvm
                .invoke_virtual(
                    &screen,
                    "javax/microedition/lcdui/Displayable",
                    "setTitle",
                    "(Ljava/lang/String;)V",
                    (None,),
                )
                .await?;
            let detached_height = viewport_height(&jvm, &screen).await?;
            assert!(detached_height > fully_decorated_height);
            assert_eq!(jvm.get_field::<i32>(&screen, "callbackCount", "I").await?, 3);

            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    (screen.clone(),),
                )
                .await?;
            assert_eq!(jvm.get_field::<i32>(&screen, "callbackCount", "I").await?, 4);
            assert_eq!(jvm.get_field::<i32>(&screen, "lastHeight", "I").await?, detached_height);

            Ok(())
        })
    }

    #[test]
    fn size_changed_coalesces_callback_decoration_mutation() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let mut screen: ClassInstanceRef<ViewportScreen> = jvm.new_class("javax/microedition/lcdui/TestViewportScreen", "()V", ()).await?.into();
            jvm.put_field(&mut screen, "mutateOnFirstCallback", "Z", true).await?;
            let display: ClassInstanceRef<Display> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();

            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    (screen.clone(),),
                )
                .await?;

            assert_eq!(jvm.get_field::<i32>(&screen, "callbackCount", "I").await?, 2);
            assert_eq!(jvm.get_field::<i32>(&screen, "maximumCallbackDepth", "I").await?, 1);
            assert_eq!(jvm.get_field::<i32>(&screen, "callbackDepth", "I").await?, 0);
            assert_eq!(jvm.get_field::<i32>(&screen, "lastWidth", "I").await?, 320);
            let current_height = viewport_height(&jvm, &screen).await?;
            assert_eq!(jvm.get_field::<i32>(&screen, "lastHeight", "I").await?, current_height);
            assert!(current_height < 240);

            Ok(())
        })
    }

    #[test]
    fn screen_paint_uses_content_coordinates_and_renders_chrome_bands() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let screen: ClassInstanceRef<ViewportScreen> = jvm.new_class("javax/microedition/lcdui/TestViewportScreen", "()V", ()).await?.into();
            set_title(
                &jvm,
                &screen,
                "First title line\nA second title line that is long enough to wrap onto another visible title row",
            )
            .await?;
            let command = make_command(&jvm, "Select", 4, 0).await?;
            let _: () = jvm
                .invoke_virtual(
                    &screen,
                    "javax/microedition/lcdui/Displayable",
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    (command,),
                )
                .await?;

            let display: ClassInstanceRef<Display> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
            let mut graphics: ClassInstanceRef<Graphics> = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "getScreenGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    (),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xaa22aa,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 320, 240))
                .await?;

            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    (screen.clone(),),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                .await?;

            let translate_x: i32 = jvm.get_field(&screen, "translateX", "I").await?;
            let translate_y: i32 = jvm.get_field(&screen, "translateY", "I").await?;
            let clip_width: i32 = jvm.get_field(&screen, "clipWidth", "I").await?;
            let clip_height: i32 = jvm.get_field(&screen, "clipHeight", "I").await?;
            assert_eq!(translate_x, 0);
            assert!(translate_y >= 40, "title newline/wrap must reserve multiple rows");
            assert_eq!(jvm.get_field::<i32>(&screen, "clipX", "I").await?, 0);
            assert_eq!(jvm.get_field::<i32>(&screen, "clipY", "I").await?, 0);
            assert_eq!(clip_width, 320);
            assert_eq!(clip_height, viewport_height(&jvm, &screen).await?);

            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getTranslateX", "()I", ())
                    .await?,
                0
            );
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getTranslateY", "()I", ())
                    .await?,
                0
            );
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
                    .await?,
                240
            );
            assert_eq!(
                jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
                    .await?,
                320
            );

            let image_ref = Graphics::image(&jvm, &mut graphics).await?;
            let image = Image::image(&jvm, &image_ref).await?;
            let rgb = |x, y| {
                let color = image.get_pixel(x, y);
                (color.r, color.g, color.b)
            };
            assert_eq!(rgb(0, translate_y), (0xcc, 0x11, 0x33));
            assert_ne!(rgb(0, translate_y - 1), (0xcc, 0x11, 0x33));
            assert_eq!(rgb(319, translate_y + clip_height - 1), (0x11, 0x88, 0xcc));
            assert_eq!(rgb(100, translate_y + 20), (0xff, 0xff, 0xff), "Screen must clear stale content pixels");
            assert_ne!(rgb(319, 0), (0xff, 0xff, 0xff), "title band must be painted in screen coordinates");
            assert_ne!(rgb(319, 0), (0xaa, 0x22, 0xaa), "title band must replace stale pixels");
            assert_ne!(
                rgb(319, translate_y - 1),
                (0xff, 0xff, 0xff),
                "title band must remain outside the content clip"
            );
            assert_ne!(rgb(319, translate_y - 1), (0xaa, 0x22, 0xaa), "title band must replace stale pixels");
            assert_ne!(
                rgb(319, 239),
                (0xff, 0xff, 0xff),
                "softkey band must be painted at the bottom of the screen"
            );
            assert_ne!(rgb(319, 239), (0xaa, 0x22, 0xaa), "softkey band must replace stale pixels");

            let bright_title_pixels = (0..translate_y - 16)
                .flat_map(|y| (0..200).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    let (r, g, b) = rgb(x, y);
                    r > 200 && g > 200 && b > 200
                })
                .count();
            assert!(bright_title_pixels > 20, "wrapped title rows must contain visible glyph pixels");

            Ok(())
        })
    }

    #[test]
    fn base_displayable_commands_use_direct_softkeys_and_ordered_options() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let screen: ClassInstanceRef<ViewportScreen> = jvm.new_class("javax/microedition/lcdui/TestViewportScreen", "()V", ()).await?.into();
            let listener: ClassInstanceRef<RecordingCommandListener> = jvm
                .new_class("javax/microedition/lcdui/TestRecordingCommandListener", "()V", ())
                .await?
                .into();
            let listener_ref: ClassInstanceRef<CommandListener> = JavaValue::from(listener.clone()).into();
            let _: () = jvm
                .invoke_virtual(
                    &screen,
                    "javax/microedition/lcdui/Displayable",
                    "setCommandListener",
                    "(Ljavax/microedition/lcdui/CommandListener;)V",
                    (listener_ref,),
                )
                .await?;

            let low_priority = make_command(&jvm, "Later", 1, 5).await?;
            let first_tied = make_command(&jvm, "First", 4, 1).await?;
            let second_tied = make_command(&jvm, "Second", 5, 1).await?;
            let back = make_command(&jvm, "Back", 2, 2).await?;
            for command in [&low_priority, &first_tied, &second_tied, &back] {
                let _: () = jvm
                    .invoke_virtual(
                        &screen,
                        "javax/microedition/lcdui/Displayable",
                        "addCommand",
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command.clone(),),
                    )
                    .await?;
            }

            let display: ClassInstanceRef<Display> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    (screen.clone(),),
                )
                .await?;

            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::RIGHT_SOFT_KEY).await?;
            assert_eq!(jvm.get_field::<i32>(&listener, "count", "I").await?, 1);
            let dispatched: ClassInstanceRef<Command> = jvm.get_field(&listener, "lastCommand", "Ljavax/microedition/lcdui/Command;").await?;
            assert_eq!(dispatched.identity(), back.identity());
            let target: ClassInstanceRef<Displayable> = jvm
                .get_field(&listener, "lastDisplayable", "Ljavax/microedition/lcdui/Displayable;")
                .await?;
            assert_eq!(target.identity(), screen.identity());

            for event_type in [KeyboardEventType::KeyRepeated, KeyboardEventType::KeyReleased] {
                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "handleKeyEvent",
                        "(II)V",
                        (event_type as i32, MIDPKeyCode::RIGHT_SOFT_KEY as i32),
                    )
                    .await?;
            }
            assert_eq!(jvm.get_field::<i32>(&listener, "count", "I").await?, 1);
            assert_eq!(jvm.get_field::<i32>(&screen, "keyCount", "I").await?, 0);

            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::KEY_NUM1).await?;
            assert_eq!(jvm.get_field::<i32>(&screen, "keyCount", "I").await?, 1);
            assert_eq!(jvm.get_field::<i32>(&screen, "lastKeyCode", "I").await?, MIDPKeyCode::KEY_NUM1 as i32);

            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::LEFT_SOFT_KEY).await?;
            assert_eq!(jvm.get_field::<i32>(&listener, "count", "I").await?, 1);

            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                .await?;
            let mut graphics: ClassInstanceRef<Graphics> = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "getScreenGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    (),
                )
                .await?;
            let image_ref = Graphics::image(&jvm, &mut graphics).await?;
            let image = Image::image(&jvm, &image_ref).await?;
            let menu_band_non_white = (160..222)
                .flat_map(|y| (0..160).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    let color = image.get_pixel(x, y);
                    (color.r, color.g, color.b) != (0xff, 0xff, 0xff)
                })
                .count();
            assert!(menu_band_non_white > 100, "open options menu must render above the softkey bar");

            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::DOWN).await?;
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::FIRE).await?;
            assert_eq!(jvm.get_field::<i32>(&listener, "count", "I").await?, 2);
            let dispatched: ClassInstanceRef<Command> = jvm.get_field(&listener, "lastCommand", "Ljavax/microedition/lcdui/Command;").await?;
            assert_eq!(
                dispatched.identity(),
                second_tied.identity(),
                "equal priorities must retain registration order"
            );
            assert_eq!(
                jvm.get_field::<i32>(&screen, "keyCount", "I").await?,
                1,
                "menu keys must not leak to the Displayable"
            );

            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::LEFT_SOFT_KEY).await?;
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::RIGHT_SOFT_KEY).await?;
            assert_eq!(
                jvm.get_field::<i32>(&listener, "count", "I").await?,
                2,
                "right softkey must cancel an open menu"
            );

            for command in [&low_priority, &second_tied, &back] {
                let _: () = jvm
                    .invoke_virtual(
                        &screen,
                        "javax/microedition/lcdui/Displayable",
                        "removeCommand",
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command.clone(),),
                    )
                    .await?;
            }
            send_key(&jvm, &display, KeyboardEventType::KeyPressed, MIDPKeyCode::LEFT_SOFT_KEY).await?;
            assert_eq!(jvm.get_field::<i32>(&listener, "count", "I").await?, 3);
            let dispatched: ClassInstanceRef<Command> = jvm.get_field(&listener, "lastCommand", "Ljavax/microedition/lcdui/Command;").await?;
            assert_eq!(dispatched.identity(), first_tied.identity());

            Ok(())
        })
    }
}
