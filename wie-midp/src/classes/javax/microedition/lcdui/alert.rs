use alloc::{string::String as RustString, vec, vec::Vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{lang::String, util::Vector};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{
    AlertType, Command, CommandListener, Display, Displayable, Font, Gauge, Graphics, Image, Item, ItemCommandListener,
};
use crate::classes::net::wie::{KeyboardEventType, MIDPKeyCode};

const CONTENT_GAP: i32 = 4;
const LEFT_TOP: i32 = 4 | 16;

struct AlertContentLayout {
    image: ClassInstanceRef<Image>,
    image_width: i32,
    image_height: i32,
    text_lines: Vec<RustString>,
    text_y: i32,
    indicator: ClassInstanceRef<Gauge>,
    indicator_y: i32,
    indicator_height: i32,
    height: i32,
}

// class javax.microedition.lcdui.Alert
pub struct Alert;

impl Alert {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Alert",
            parent_class: Some("javax/microedition/lcdui/Screen"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;Ljavax/microedition/lcdui/Image;Ljavax/microedition/lcdui/AlertType;)V",
                    Self::init_with_contents,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getDefaultTimeout", "()I", Self::get_default_timeout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getTimeout", "()I", Self::get_timeout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTimeout", "(I)V", Self::set_timeout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getType",
                    "()Ljavax/microedition/lcdui/AlertType;",
                    Self::get_type,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setType",
                    "(Ljavax/microedition/lcdui/AlertType;)V",
                    Self::set_type,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getImage",
                    "()Ljavax/microedition/lcdui/Image;",
                    Self::get_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setImage",
                    "(Ljavax/microedition/lcdui/Image;)V",
                    Self::set_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getIndicator",
                    "()Ljavax/microedition/lcdui/Gauge;",
                    Self::get_indicator,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setIndicator",
                    "(Ljavax/microedition/lcdui/Gauge;)V",
                    Self::set_indicator,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::add_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "removeCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::remove_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setCommandListener",
                    "(Ljavax/microedition/lcdui/CommandListener;)V",
                    Self::set_command_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("decorationChanged", "()V", Self::decoration_changed, MethodAccessFlags::empty()),
                JavaMethodProto::new("getCommandCount", "()I", Self::get_command_count, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "getCommandAt",
                    "(I)Ljavax/microedition/lcdui/Command;",
                    Self::get_command_at,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("dispatchCommandAt", "(I)V", Self::dispatch_command_at, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "setDisplay",
                    "(Ljavax/microedition/lcdui/Display;)V",
                    Self::set_display,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "checkItemMutation",
                    "(Ljavax/microedition/lcdui/Item;)V",
                    Self::check_item_mutation,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "itemInvalidated",
                    "(Ljavax/microedition/lcdui/Item;Z)V",
                    Self::item_invalidated,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "handlePaintEvent",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::handle_paint_event,
                    MethodAccessFlags::empty(),
                ),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "FOREVER",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "DISMISS_COMMAND",
                    "Ljavax/microedition/lcdui/Command;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("type", "Ljavax/microedition/lcdui/AlertType;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("image", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("displayImage", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("indicator", "Ljavax/microedition/lcdui/Gauge;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("timeout", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("nextDisplayable", "Ljavax/microedition/lcdui/Displayable;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("scrollY", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/Alert", "FOREVER", "I", -2).await?;
        let short_label = JavaLangString::from_rust_string(jvm, "").await?;
        let long_label = ClassInstanceRef::<String>::new(None);
        let dismiss_command = jvm
            .new_class(
                "javax/microedition/lcdui/Command",
                "(Ljava/lang/String;Ljava/lang/String;II)V",
                (short_label, long_label, 4, 0),
            )
            .await?;
        jvm.put_static_field(
            "javax/microedition/lcdui/Alert",
            "DISMISS_COMMAND",
            "Ljavax/microedition/lcdui/Command;",
            dismiss_command,
        )
        .await
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::<init>({this:?}, {title:?})");

        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Alert",
            "<init>",
            "(Ljava/lang/String;Ljava/lang/String;Ljavax/microedition/lcdui/Image;Ljavax/microedition/lcdui/AlertType;)V",
            (
                title,
                ClassInstanceRef::<String>::new(None),
                ClassInstanceRef::<Image>::new(None),
                ClassInstanceRef::<AlertType>::new(None),
            ),
        )
        .await
    }

    async fn init_with_contents(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
        alert_type: ClassInstanceRef<AlertType>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::<init>({this:?}, {title:?}, {text:?}, {image:?}, {alert_type:?})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await?;
        let display_image = Self::snapshot(jvm, &image).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "type", "Ljavax/microedition/lcdui/AlertType;", alert_type)
            .await?;
        jvm.put_field(&mut this, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(&mut this, "displayImage", "Ljavax/microedition/lcdui/Image;", display_image)
            .await?;
        jvm.put_field(&mut this, "timeout", "I", 2000).await?;
        jvm.put_field(&mut this, "scrollY", "I", 0).await?;
        jvm.invoke_virtual(
            &this,
            "javax/microedition/lcdui/Displayable",
            "setTitle",
            "(Ljava/lang/String;)V",
            (title,),
        )
        .await
    }

    async fn snapshot(jvm: &Jvm, image: &ClassInstanceRef<Image>) -> JvmResult<ClassInstanceRef<Image>> {
        if image.is_null() {
            Ok(ClassInstanceRef::new(None))
        } else {
            jvm.invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(Ljavax/microedition/lcdui/Image;)Ljavax/microedition/lcdui/Image;",
                (image.clone(),),
            )
            .await
        }
    }

    async fn content_layout(jvm: &Jvm, context: &mut WieJvmContext, this: &ClassInstanceRef<Self>, width: i32) -> JvmResult<AlertContentLayout> {
        let width = width.max(0);
        let image: ClassInstanceRef<Image> = jvm.get_field(this, "displayImage", "Ljavax/microedition/lcdui/Image;").await?;
        let (image_width, image_height): (i32, i32) = if image.is_null() {
            (0, 0)
        } else {
            (
                jvm.invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
                    .await?,
                jvm.invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
                    .await?,
            )
        };
        let image_width = image_width.max(0);
        let image_height = image_height.max(0);
        let mut height: i32 = image_height;

        let text: ClassInstanceRef<String> = jvm.get_field(this, "text", "Ljava/lang/String;").await?;
        let text_lines = if text.is_null() {
            Vec::new()
        } else {
            let text = JavaLangString::to_rust_string(jvm, &text).await?;
            if text.is_empty() {
                Vec::new()
            } else {
                Font::wrap(context.system().platform().font(), &text, Some(width.max(1)))
            }
        };
        let text_y = if text_lines.is_empty() {
            height
        } else {
            if height > 0 {
                height = height.saturating_add(CONTENT_GAP);
            }
            let text_y = height;
            height = height.saturating_add((text_lines.len() as i32).saturating_mul(Font::HEIGHT));
            text_y
        };

        let indicator: ClassInstanceRef<Gauge> = jvm.get_field(this, "indicator", "Ljavax/microedition/lcdui/Gauge;").await?;
        let indicator_height: i32 = if indicator.is_null() {
            0
        } else {
            let height: i32 = jvm
                .invoke_virtual(&indicator, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (width,))
                .await?;
            height.max(0)
        };
        let indicator_y = if indicator_height == 0 {
            height
        } else {
            if height > 0 {
                height = height.saturating_add(CONTENT_GAP);
            }
            let indicator_y = height;
            height = height.saturating_add(indicator_height);
            indicator_y
        };

        Ok(AlertContentLayout {
            image,
            image_width,
            image_height,
            text_lines,
            text_y,
            indicator,
            indicator_y,
            indicator_height,
            height,
        })
    }

    async fn effective_timeout(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let width: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await?;
        let viewport_height: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;
        let viewport_height = viewport_height.max(0);
        let content = Self::content_layout(jvm, context, &this, width).await?;
        let maximum_scroll = content.height.saturating_sub(viewport_height).max(0);
        let scroll: i32 = jvm.get_field(&this, "scrollY", "I").await?;
        let clamped_scroll = scroll.clamp(0, maximum_scroll);
        if clamped_scroll != scroll {
            jvm.put_field(&mut this, "scrollY", "I", clamped_scroll).await?;
        }

        let application_command_count: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getCommandCount", "()I", ())
            .await?;
        if maximum_scroll > 0 || application_command_count > 1 {
            Ok(-2)
        } else {
            jvm.get_field(&this, "timeout", "I").await
        }
    }

    async fn invalidate_current(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, request_repaint: bool) -> JvmResult<()> {
        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        if display.is_null() {
            let _ = Self::effective_timeout(jvm, context, this).await?;
            return Ok(());
        }

        Display::alert_changed(jvm, context, display).await?;
        if request_repaint {
            let _: () = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
                .await?;
        }
        Ok(())
    }

    async fn get_default_timeout(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(2000)
    }

    async fn get_timeout(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Self::effective_timeout(jvm, context, this).await
    }

    async fn set_timeout(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, timeout: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::setTimeout({this:?}, {timeout})");

        if timeout <= 0 && timeout != -2 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid Alert timeout").await);
        }
        jvm.put_field(&mut this, "timeout", "I", timeout).await?;
        Self::invalidate_current(jvm, context, this, true).await
    }

    async fn get_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<AlertType>> {
        jvm.get_field(&this, "type", "Ljavax/microedition/lcdui/AlertType;").await
    }

    async fn set_type(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        alert_type: ClassInstanceRef<AlertType>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::setType({this:?}, {alert_type:?})");
        jvm.put_field(&mut this, "type", "Ljavax/microedition/lcdui/AlertType;", alert_type)
            .await?;
        Self::invalidate_current(jvm, context, this, true).await
    }

    async fn get_string(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "text", "Ljava/lang/String;").await
    }

    async fn set_string(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Alert::setString({this:?}, {text:?})");
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        Self::invalidate_current(jvm, context, this, true).await
    }

    async fn get_image(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Image>> {
        jvm.get_field(&this, "image", "Ljavax/microedition/lcdui/Image;").await
    }

    async fn set_image(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        let display_image = Self::snapshot(jvm, &image).await?;
        jvm.put_field(&mut this, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(&mut this, "displayImage", "Ljavax/microedition/lcdui/Image;", display_image)
            .await?;
        Self::invalidate_current(jvm, context, this, true).await
    }

    async fn get_indicator(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Gauge>> {
        jvm.get_field(&this, "indicator", "Ljavax/microedition/lcdui/Gauge;").await
    }

    async fn set_indicator(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        mut indicator: ClassInstanceRef<Gauge>,
    ) -> JvmResult<()> {
        let mut old_indicator: ClassInstanceRef<Gauge> = jvm.get_field(&this, "indicator", "Ljavax/microedition/lcdui/Gauge;").await?;
        if (old_indicator.is_null() && indicator.is_null())
            || (!old_indicator.is_null() && !indicator.is_null() && old_indicator.identity() == indicator.identity())
        {
            return Ok(());
        }

        if !indicator.is_null() {
            let interactive: bool = jvm.get_field(&indicator, "interactive", "Z").await?;
            let owner: ClassInstanceRef<Displayable> = jvm.get_field(&indicator, "owner", "Ljavax/microedition/lcdui/Displayable;").await?;
            let commands: ClassInstanceRef<Vector> = jvm.get_field(&indicator, "commands", "Ljava/util/Vector;").await?;
            let command_count: i32 = jvm.invoke_virtual(&commands, "java/util/Vector", "size", "()I", ()).await?;
            let listener: ClassInstanceRef<ItemCommandListener> = jvm
                .get_field(&indicator, "itemCommandListener", "Ljavax/microedition/lcdui/ItemCommandListener;")
                .await?;
            let label: ClassInstanceRef<String> = jvm.get_field(&indicator, "label", "Ljava/lang/String;").await?;
            let preferred_width: i32 = jvm.get_field(&indicator, "preferredWidth", "I").await?;
            let preferred_height: i32 = jvm.get_field(&indicator, "preferredHeight", "I").await?;
            let layout: i32 = jvm.get_field(&indicator, "layout", "I").await?;
            if interactive
                || !owner.is_null()
                || command_count != 0
                || !listener.is_null()
                || !label.is_null()
                || preferred_width != -1
                || preferred_height != -1
                || layout != 0
            {
                return Err(jvm
                    .exception("java/lang/IllegalArgumentException", "Gauge cannot be used as an Alert indicator")
                    .await);
            }
        }

        if !old_indicator.is_null() {
            jvm.put_field(
                &mut old_indicator,
                "owner",
                "Ljavax/microedition/lcdui/Displayable;",
                ClassInstanceRef::<Displayable>::new(None),
            )
            .await?;
        }
        if !indicator.is_null() {
            jvm.put_field(&mut indicator, "owner", "Ljavax/microedition/lcdui/Displayable;", this.clone())
                .await?;
        }
        jvm.put_field(&mut this, "indicator", "Ljavax/microedition/lcdui/Gauge;", indicator)
            .await?;
        Self::invalidate_current(jvm, context, this, true).await
    }

    async fn add_command(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<()> {
        let dismiss: ClassInstanceRef<Command> = jvm
            .get_static_field("javax/microedition/lcdui/Alert", "DISMISS_COMMAND", "Ljavax/microedition/lcdui/Command;")
            .await?;
        if !command.is_null() && command.identity() == dismiss.identity() {
            return Ok(());
        }
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "addCommand",
            "(Ljavax/microedition/lcdui/Command;)V",
            (command,),
        )
        .await
    }

    async fn remove_command(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        let dismiss: ClassInstanceRef<Command> = jvm
            .get_static_field("javax/microedition/lcdui/Alert", "DISMISS_COMMAND", "Ljavax/microedition/lcdui/Command;")
            .await?;
        if !command.is_null() && command.identity() == dismiss.identity() {
            return Ok(());
        }
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "removeCommand",
            "(Ljavax/microedition/lcdui/Command;)V",
            (command,),
        )
        .await
    }

    async fn set_command_listener(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<CommandListener>,
    ) -> JvmResult<()> {
        let listener_result: JvmResult<()> = jvm
            .invoke_special(
                &this,
                "javax/microedition/lcdui/Displayable",
                "setCommandListener",
                "(Ljavax/microedition/lcdui/CommandListener;)V",
                (listener,),
            )
            .await;
        let invalidation_result = Self::invalidate_current(jvm, context, this, true).await;
        listener_result?;
        invalidation_result
    }

    async fn decoration_changed(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let decoration_result: JvmResult<()> = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "decorationChanged", "()V", ())
            .await;
        let invalidation_result = Self::invalidate_current(jvm, context, this, false).await;
        decoration_result?;
        invalidation_result
    }

    async fn get_command_count(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let count: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getCommandCount", "()I", ())
            .await?;
        Ok(count.max(1))
    }

    async fn get_command_at(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        index: i32,
    ) -> JvmResult<ClassInstanceRef<Command>> {
        let count: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getCommandCount", "()I", ())
            .await?;
        if count == 0 && index == 0 {
            return jvm
                .get_static_field("javax/microedition/lcdui/Alert", "DISMISS_COMMAND", "Ljavax/microedition/lcdui/Command;")
                .await;
        }

        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "getCommandAt",
            "(I)Ljavax/microedition/lcdui/Command;",
            (index,),
        )
        .await
    }

    async fn dispatch_command_at(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        let command: ClassInstanceRef<Command> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/Displayable",
                "getCommandAt",
                "(I)Ljavax/microedition/lcdui/Command;",
                (index,),
            )
            .await?;
        let listener: ClassInstanceRef<CommandListener> = jvm
            .get_field(&this, "commandListener", "Ljavax/microedition/lcdui/CommandListener;")
            .await?;
        if !listener.is_null() {
            return jvm
                .invoke_virtual(
                    &listener,
                    "javax/microedition/lcdui/CommandListener",
                    "commandAction",
                    "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V",
                    (command, this),
                )
                .await;
        }

        let next: ClassInstanceRef<Displayable> = jvm.get_field(&this, "nextDisplayable", "Ljavax/microedition/lcdui/Displayable;").await?;
        jvm.put_field(
            &mut this,
            "nextDisplayable",
            "Ljavax/microedition/lcdui/Displayable;",
            ClassInstanceRef::<Displayable>::new(None),
        )
        .await?;
        let display: ClassInstanceRef<Display> = jvm.get_field(&this, "currentDisplay", "Ljavax/microedition/lcdui/Display;").await?;
        if display.is_null() {
            return Ok(());
        }

        let current: ClassInstanceRef<Displayable> = jvm
            .invoke_virtual(
                &display,
                "javax/microedition/lcdui/Display",
                "getCurrent",
                "()Ljavax/microedition/lcdui/Displayable;",
                (),
            )
            .await?;
        if !current.is_null() && current.identity() == this.identity() {
            Display::transition(jvm, context, display, next).await?;
        }

        Ok(())
    }

    async fn set_display(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        display: ClassInstanceRef<Display>,
    ) -> JvmResult<()> {
        if display.is_null() {
            jvm.put_field(
                &mut this,
                "nextDisplayable",
                "Ljavax/microedition/lcdui/Displayable;",
                ClassInstanceRef::<Displayable>::new(None),
            )
            .await?;
        }
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "setDisplay",
            "(Ljavax/microedition/lcdui/Display;)V",
            (display,),
        )
        .await
    }

    async fn item_invalidated(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        _item: ClassInstanceRef<Item>,
        _layout_changed: bool,
    ) -> JvmResult<()> {
        Self::invalidate_current(jvm, context, this, true).await
    }

    async fn handle_key_event(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        let pressed = event_type == KeyboardEventType::KeyPressed as i32;
        let repeated = event_type == KeyboardEventType::KeyRepeated as i32;
        if (pressed || repeated) && (code == MIDPKeyCode::UP as i32 || code == MIDPKeyCode::DOWN as i32) {
            let width: i32 = jvm
                .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
                .await?;
            let viewport_height: i32 = jvm
                .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                .await?;
            let viewport_height = viewport_height.max(0);
            let content = Self::content_layout(jvm, context, &this, width).await?;
            let maximum_scroll = content.height.saturating_sub(viewport_height).max(0);
            let scroll: i32 = jvm.get_field(&this, "scrollY", "I").await?;
            let scroll = scroll.clamp(0, maximum_scroll);
            let step = viewport_height.max(1);
            let next_scroll = if code == MIDPKeyCode::UP as i32 {
                scroll.saturating_sub(step).max(0)
            } else {
                scroll.saturating_add(step).min(maximum_scroll)
            };
            if next_scroll != scroll {
                jvm.put_field(&mut this, "scrollY", "I", next_scroll).await?;
                return jvm
                    .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
                    .await;
            }
        }

        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "handleKeyEvent",
            "(II)V",
            (event_type, code),
        )
        .await
    }

    async fn handle_paint_event(
        jvm: &Jvm,
        context: &mut WieJvmContext,
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
        let width: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await?;
        let width = width.max(0);
        let viewport_height: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;
        let viewport_height = viewport_height.max(0);
        let content = Self::content_layout(jvm, context, &this, width).await?;
        let maximum_scroll = content.height.saturating_sub(viewport_height).max(0);
        let scroll: i32 = jvm.get_field(&this, "scrollY", "I").await?;
        let scroll = scroll.clamp(0, maximum_scroll);
        jvm.put_field(&mut this, "scrollY", "I", scroll).await?;

        if !content.image.is_null() && content.image_width > 0 && content.image_height > 0 {
            let image_x = ((width - content.image_width) / 2).max(0);
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawImage",
                    "(Ljavax/microedition/lcdui/Image;III)V",
                    (content.image, image_x, -scroll, LEFT_TOP),
                )
                .await?;
        }

        if !content.text_lines.is_empty() {
            let font: ClassInstanceRef<Font> = jvm
                .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "setFont",
                    "(Ljavax/microedition/lcdui/Font;)V",
                    (font,),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0,))
                .await?;
            for (index, line) in content.text_lines.iter().enumerate() {
                let y = content.text_y + index as i32 * Font::HEIGHT - scroll;
                if y.saturating_add(Font::HEIGHT) <= 0 || y >= viewport_height {
                    continue;
                }
                let line = JavaLangString::from_rust_string(jvm, line).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &graphics,
                        "javax/microedition/lcdui/Graphics",
                        "drawString",
                        "(Ljava/lang/String;III)V",
                        (line, 0, y, LEFT_TOP),
                    )
                    .await?;
            }
        }

        if !content.indicator.is_null() && content.indicator_height > 0 {
            let _: () = jvm
                .invoke_virtual(
                    &content.indicator,
                    "javax/microedition/lcdui/Item",
                    "paintItem",
                    "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                    (graphics, 0, content.indicator_y - scroll, width, content.indicator_height, false),
                )
                .await?;
        }

        Ok(())
    }

    async fn check_item_mutation(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        _this: ClassInstanceRef<Self>,
        _item: ClassInstanceRef<Item>,
    ) -> JvmResult<()> {
        Err(jvm
            .exception("java/lang/IllegalStateException", "Alert indicator cannot be modified")
            .await)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec};

    use jvm::{Array, ClassInstanceRef, JavaError, JavaValue, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::String;

    use test_utils::{TestClock, TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_backend::{Event, System};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::{
        classes::{
            javax::microedition::{
                lcdui::{Alert, AlertType, Command, CommandListener, Display, Displayable, Gauge, Graphics, Image},
                midlet::MIDlet,
            },
            net::wie::{KeyboardEventType, MIDPKeyCode},
        },
        get_protos,
    };

    struct RecordingAlertCommandListener;
    struct AlertTestMidlet;

    impl RecordingAlertCommandListener {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/lcdui/TestAlertCommandListener",
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
            Err(jvm.exception("java/lang/RuntimeException", "listener failure").await)
        }
    }

    impl AlertTestMidlet {
        fn as_proto() -> WieJavaClassProto {
            JavaClassProto {
                name: "javax/microedition/midlet/TestAlertMidlet",
                parent_class: Some("javax/microedition/midlet/MIDlet"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("startApp", "()V", Self::start_app, MethodAccessFlags::PROTECTED),
                ],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/midlet/MIDlet", "<init>", "()V", ()).await
        }

        async fn start_app(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<()> {
            Ok(())
        }
    }

    fn test_protos() -> Box<[Box<[WieJavaClassProto]>]> {
        Box::new([
            get_protos().into(),
            [RecordingAlertCommandListener::as_proto(), AlertTestMidlet::as_proto()].into(),
        ])
    }

    async fn new_alert(jvm: &Jvm, text: &str) -> JvmResult<ClassInstanceRef<Alert>> {
        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Alert",
                "(Ljava/lang/String;Ljava/lang/String;Ljavax/microedition/lcdui/Image;Ljavax/microedition/lcdui/AlertType;)V",
                (
                    ClassInstanceRef::<String>::new(None),
                    JavaLangString::from_rust_string(jvm, text).await?,
                    ClassInstanceRef::<Image>::new(None),
                    ClassInstanceRef::<AlertType>::new(None),
                ),
            )
            .await?
            .into())
    }

    async fn new_form(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Displayable>> {
        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Form",
                "(Ljava/lang/String;)V",
                (ClassInstanceRef::<String>::new(None),),
            )
            .await?
            .into())
    }

    async fn new_command(jvm: &Jvm, label: &str) -> JvmResult<ClassInstanceRef<Command>> {
        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Command",
                "(Ljava/lang/String;II)V",
                (JavaLangString::from_rust_string(jvm, label).await?, 4, 0),
            )
            .await?
            .into())
    }

    async fn new_gauge(jvm: &Jvm, interactive: bool, value: i32) -> JvmResult<ClassInstanceRef<Gauge>> {
        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Gauge",
                "(Ljava/lang/String;ZII)V",
                (ClassInstanceRef::<String>::new(None), interactive, 10, value),
            )
            .await?
            .into())
    }

    async fn current(jvm: &Jvm, display: &ClassInstanceRef<Display>) -> JvmResult<ClassInstanceRef<Displayable>> {
        jvm.invoke_virtual(
            display,
            "javax/microedition/lcdui/Display",
            "getCurrent",
            "()Ljavax/microedition/lcdui/Displayable;",
            (),
        )
        .await
    }

    async fn show(jvm: &Jvm, display: &ClassInstanceRef<Display>, displayable: ClassInstanceRef<Displayable>) -> JvmResult<()> {
        jvm.invoke_virtual(
            display,
            "javax/microedition/lcdui/Display",
            "setCurrent",
            "(Ljavax/microedition/lcdui/Displayable;)V",
            (displayable,),
        )
        .await
    }

    async fn test_midlet_display(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Display>> {
        let midlet: ClassInstanceRef<MIDlet> = jvm.new_class("javax/microedition/midlet/TestAlertMidlet", "()V", ()).await?.into();
        MIDlet::display(jvm, &midlet).await
    }

    async fn pump_backend_queue(jvm: &Jvm, system: &System) -> JvmResult<()> {
        // A FIFO sentinel returns control after preceding timers have been processed.
        system.event_queue().push(Event::Notify {
            r#type: 731,
            param1: 19,
            param2: 23,
        });
        let queue = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;
        let event: ClassInstanceRef<Array<i32>> = jvm.instantiate_array("I", 4).await?.into();
        let _: () = jvm
            .invoke_virtual(&queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
            .await?;
        assert_eq!(jvm.load_array::<i32>(&event, 0, 4).await?, [1000, 731, 19, 23]);
        Ok(())
    }

    #[test]
    fn alert_renders_snapshots_indicator_updates_and_scrolls_overflow() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let mut display = test_midlet_display(&jvm).await?;
            jvm.put_field(&mut display, "width", "I", 60).await?;
            jvm.put_field(&mut display, "height", "I", 80).await?;
            let source: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (10, 6),
                )
                .await?;
            let graphics: ClassInstanceRef<Graphics> = jvm
                .invoke_virtual(
                    &source,
                    "javax/microedition/lcdui/Image",
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    (),
                )
                .await?;
            let alert = new_alert(&jvm, "status").await?;
            for color in [0xcc2233, 0x22bb55] {
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (color,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 10, 6))
                    .await?;
                if color == 0xcc2233 {
                    let _: () = jvm
                        .invoke_virtual(
                            &alert,
                            "javax/microedition/lcdui/Alert",
                            "setImage",
                            "(Ljavax/microedition/lcdui/Image;)V",
                            (source.clone(),),
                        )
                        .await?;
                }
            }
            let indicator = new_gauge(&jvm, false, 2).await?;
            let _: () = jvm
                .invoke_virtual(
                    &alert,
                    "javax/microedition/lcdui/Alert",
                    "setIndicator",
                    "(Ljavax/microedition/lcdui/Gauge;)V",
                    (indicator.clone(),),
                )
                .await?;
            show(&jvm, &display, JavaValue::from(alert.clone()).into()).await?;
            let dismiss: ClassInstanceRef<Command> = jvm
                .get_static_field("javax/microedition/lcdui/Alert", "DISMISS_COMMAND", "Ljavax/microedition/lcdui/Command;")
                .await?;
            let effective: ClassInstanceRef<Command> = jvm
                .invoke_virtual(
                    &alert,
                    "javax/microedition/lcdui/Displayable",
                    "getCommandAt",
                    "(I)Ljavax/microedition/lcdui/Command;",
                    (0,),
                )
                .await?;
            assert_eq!(dismiss.identity(), effective.identity());
            let label = jvm
                .invoke_virtual(&dismiss, "javax/microedition/lcdui/Command", "getLabel", "()Ljava/lang/String;", ())
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &label).await?, "");
            let mut screen_graphics = Display::screen_graphics(&jvm, &display).await?;
            let screen_image = Graphics::image(&jvm, &mut screen_graphics).await?;
            let mut initial_fill = 0;
            for value in [2, 8] {
                let _: () = jvm
                    .invoke_virtual(&indicator, "javax/microedition/lcdui/Gauge", "setValue", "(I)V", (value,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                    .await?;
                let image = Image::image(&jvm, &screen_image).await?;
                assert!(
                    (64..76).any(|y| (4..20).any(|x| {
                        let pixel = image.get_pixel(x, y);
                        (pixel.r, pixel.g, pixel.b) != (0x26, 0x37, 0x46)
                    })),
                    "implicit dismissal must have a visible softkey caption"
                );
                let (mut image_y, mut text_y, mut gauge_y) = (None, None, None);
                let mut fill = 0;
                for y in 0..64 {
                    for x in 0..60 {
                        let color = image.get_pixel(x, y);
                        match (color.r, color.g, color.b) {
                            (0xcc, 0x22, 0x33) => image_y = Some(image_y.unwrap_or(y)),
                            (0, 0, 0) => text_y = Some(text_y.unwrap_or(y)),
                            (0x2f, 0x7e, 0xb8) => {
                                gauge_y = Some(gauge_y.unwrap_or(y));
                                fill += 1;
                            }
                            (0x22, 0xbb, 0x55) => panic!("Alert must paint the snapshot, not the mutated source"),
                            _ => {}
                        }
                    }
                }
                assert!(image_y.unwrap() < text_y.unwrap() && text_y.unwrap() < gauge_y.unwrap());
                if value == 2 {
                    initial_fill = fill;
                } else {
                    assert!(fill > initial_fill, "indicator mutation must reach the rendered buffer");
                }
            }

            let long_text = JavaLangString::from_rust_string(&jvm, &"line\n".repeat(20)).await?;
            let _: () = jvm
                .invoke_virtual(
                    &alert,
                    "javax/microedition/lcdui/Alert",
                    "setString",
                    "(Ljava/lang/String;)V",
                    (long_text,),
                )
                .await?;
            let timeout: i32 = jvm
                .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "getTimeout", "()I", ())
                .await?;
            assert_eq!(timeout, -2);
            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "handleKeyEvent",
                    "(II)V",
                    (KeyboardEventType::KeyPressed as i32, MIDPKeyCode::DOWN as i32),
                )
                .await?;
            assert!(jvm.get_field::<i32>(&alert, "scrollY", "I").await? > 0);
            let _: () = jvm
                .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                .await?;
            {
                let image = Image::image(&jvm, &screen_image).await?;
                assert!(
                    (0..64).all(|y| (0..60).all(|x| {
                        let color = image.get_pixel(x, y);
                        (color.r, color.g, color.b) != (0xcc, 0x22, 0x33)
                    })),
                    "scrolling must move the image out of the viewport"
                );
            }
            let text = JavaLangString::from_rust_string(&jvm, "short").await?;
            let _: () = jvm
                .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "setString", "(Ljava/lang/String;)V", (text,))
                .await?;
            assert_eq!(jvm.get_field::<i32>(&alert, "scrollY", "I").await?, 0);
            let timeout: i32 = jvm
                .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "getTimeout", "()I", ())
                .await?;
            assert_eq!(timeout, 2000);
            Ok(())
        })
    }

    #[test]
    fn indicator_replacement_is_atomic_restricted_and_releases_ownership() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let alert = new_alert(&jvm, "").await?;
            let form = new_form(&jvm).await?;
            let first = new_gauge(&jvm, false, 2).await?;
            let _: () = jvm
                .invoke_virtual(
                    &alert,
                    "javax/microedition/lcdui/Alert",
                    "setIndicator",
                    "(Ljavax/microedition/lcdui/Gauge;)V",
                    (first.clone(),),
                )
                .await?;
            for interactive in [true, false] {
                let candidate = new_gauge(&jvm, interactive, 2).await?;
                if !interactive {
                    let _: i32 = jvm
                        .invoke_virtual(
                            &form,
                            "javax/microedition/lcdui/Form",
                            "append",
                            "(Ljavax/microedition/lcdui/Item;)I",
                            (candidate.clone(),),
                        )
                        .await?;
                }
                let result: JvmResult<()> = jvm
                    .invoke_virtual(
                        &alert,
                        "javax/microedition/lcdui/Alert",
                        "setIndicator",
                        "(Ljavax/microedition/lcdui/Gauge;)V",
                        (candidate,),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("invalid indicator accepted: {result:?}")
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                let retained: ClassInstanceRef<Gauge> = jvm
                    .invoke_virtual(
                        &alert,
                        "javax/microedition/lcdui/Alert",
                        "getIndicator",
                        "()Ljavax/microedition/lcdui/Gauge;",
                        (),
                    )
                    .await?;
                assert_eq!(retained.identity(), first.identity());
            }
            let result: JvmResult<()> = jvm
                .invoke_virtual(
                    &first,
                    "javax/microedition/lcdui/Gauge",
                    "setLabel",
                    "(Ljava/lang/String;)V",
                    (JavaLangString::from_rust_string(&jvm, "forbidden").await?,),
                )
                .await;
            let Err(JavaError::JavaException(exception)) = result else {
                panic!("owned indicator mutation accepted: {result:?}")
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalStateException"));
            let second = new_gauge(&jvm, false, 5).await?;
            let _: () = jvm
                .invoke_virtual(
                    &alert,
                    "javax/microedition/lcdui/Alert",
                    "setIndicator",
                    "(Ljavax/microedition/lcdui/Gauge;)V",
                    (second.clone(),),
                )
                .await?;
            let _: i32 = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Form",
                    "append",
                    "(Ljavax/microedition/lcdui/Item;)I",
                    (first,),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &alert,
                    "javax/microedition/lcdui/Alert",
                    "setIndicator",
                    "(Ljavax/microedition/lcdui/Gauge;)V",
                    (ClassInstanceRef::<Gauge>::new(None),),
                )
                .await?;
            let _: i32 = jvm
                .invoke_virtual(
                    &form,
                    "javax/microedition/lcdui/Form",
                    "append",
                    "(Ljavax/microedition/lcdui/Item;)I",
                    (second,),
                )
                .await?;
            Ok(())
        })
    }

    #[test]
    fn effective_commands_control_timeout_and_default_transitions() -> Result<()> {
        run_jvm_test(test_protos(), |jvm| async move {
            let display = test_midlet_display(&jvm).await?;
            let previous = new_form(&jvm).await?;
            let alert = new_alert(&jvm, "").await?;
            let first = new_command(&jvm, "Continue").await?;
            let second = new_command(&jvm, "Cancel").await?;
            let _: () = jvm
                .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "setTimeout", "(I)V", (100,))
                .await?;
            for (method, command, timeout) in [
                ("addCommand", first.clone(), 100),
                ("addCommand", second.clone(), -2),
                ("removeCommand", second, 100),
                ("removeCommand", first, 100),
            ] {
                let _: () = jvm
                    .invoke_virtual(
                        &alert,
                        "javax/microedition/lcdui/Alert",
                        method,
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command,),
                    )
                    .await?;
                let actual_timeout: i32 = jvm
                    .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "getTimeout", "()I", ())
                    .await?;
                assert_eq!(actual_timeout, timeout);
            }
            show(&jvm, &display, previous.clone()).await?;
            show(&jvm, &display, JavaValue::from(alert.clone()).into()).await?;
            let _: () = jvm
                .invoke_virtual(
                    &display,
                    "javax/microedition/lcdui/Display",
                    "handleKeyEvent",
                    "(II)V",
                    (KeyboardEventType::KeyPressed as i32, MIDPKeyCode::LEFT_SOFT_KEY as i32),
                )
                .await?;
            assert_eq!(current(&jvm, &display).await?.identity(), previous.identity());
            Ok(())
        })
    }

    #[test]
    fn queue_timers_reschedule_mutations_and_ignore_departed_alerts() -> Result<()> {
        let clock = TestClock::new();
        run_jvm_test_with_system(
            test_protos(),
            Box::new(TestPlatform::with_clock(clock.clone())),
            move |jvm, system| async move {
                let display = test_midlet_display(&jvm).await?;
                let previous = new_form(&jvm).await?;
                let alert = new_alert(&jvm, "initial").await?;
                let indicator = new_gauge(&jvm, false, 2).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &alert,
                        "javax/microedition/lcdui/Alert",
                        "setIndicator",
                        "(Ljavax/microedition/lcdui/Gauge;)V",
                        (indicator.clone(),),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "setTimeout", "(I)V", (100,))
                    .await?;

                // Exercise decoration, Alert content, and indicator invalidation through their deadlines.
                for (index, method) in ["setTitle", "setString", "setValue", "setMaxValue"].into_iter().enumerate() {
                    let start = 1000 * (index as u64 + 1);
                    clock.set(start);
                    let _: () = jvm
                        .invoke_virtual(
                            &display,
                            "javax/microedition/lcdui/Display",
                            "setCurrent",
                            "(Ljavax/microedition/lcdui/Alert;Ljavax/microedition/lcdui/Displayable;)V",
                            (alert.clone(), previous.clone()),
                        )
                        .await?;
                    clock.set(start + 50);
                    if index < 2 {
                        let text = JavaLangString::from_rust_string(&jvm, "changed").await?;
                        let _: () = jvm
                            .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", method, "(Ljava/lang/String;)V", (text,))
                            .await?;
                    } else {
                        let _: () = jvm
                            .invoke_virtual(&indicator, "javax/microedition/lcdui/Gauge", method, "(I)V", (8,))
                            .await?;
                    }
                    for now in [start + 100, start + 149] {
                        clock.set(now);
                        pump_backend_queue(&jvm, &system).await?;
                        assert_eq!(
                            current(&jvm, &display).await?.identity(),
                            alert.identity(),
                            "{method}: stale or early timeout"
                        );
                    }
                    clock.set(start + 150);
                    pump_backend_queue(&jvm, &system).await?;
                    assert_eq!(
                        current(&jvm, &display).await?.identity(),
                        previous.identity(),
                        "{method}: replacement timer did not fire"
                    );
                }

                show(&jvm, &display, JavaValue::from(alert.clone()).into()).await?;
                let other = new_form(&jvm).await?;
                show(&jvm, &display, other.clone()).await?;
                clock.advance(100);
                pump_backend_queue(&jvm, &system).await?;
                assert_eq!(current(&jvm, &display).await?.identity(), other.identity());
                let next: ClassInstanceRef<Displayable> = jvm.get_field(&alert, "nextDisplayable", "Ljavax/microedition/lcdui/Displayable;").await?;
                assert!(next.is_null());

                let _: () = jvm
                    .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "setTimeout", "(I)V", (-2,))
                    .await?;
                show(&jvm, &display, JavaValue::from(alert.clone()).into()).await?;
                clock.advance(1000);
                pump_backend_queue(&jvm, &system).await?;
                assert_eq!(current(&jvm, &display).await?.identity(), alert.identity());
                assert!(system.event_queue().pop().is_none(), "modal Alert must not schedule a timer");
                Ok(())
            },
        )
    }

    #[test]
    fn sole_application_command_fires_once_at_the_exact_deadline_despite_listener_exception() -> Result<()> {
        let clock = TestClock::new();
        run_jvm_test_with_system(
            test_protos(),
            Box::new(TestPlatform::with_clock(clock.clone())),
            move |jvm, system| async move {
                let display = test_midlet_display(&jvm).await?;
                let previous = new_form(&jvm).await?;
                let alert = new_alert(&jvm, "custom").await?;
                let command = new_command(&jvm, "Continue").await?;
                let listener = jvm.new_class("javax/microedition/lcdui/TestAlertCommandListener", "()V", ()).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &alert,
                        "javax/microedition/lcdui/Alert",
                        "addCommand",
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command.clone(),),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &alert,
                        "javax/microedition/lcdui/Alert",
                        "setCommandListener",
                        "(Ljavax/microedition/lcdui/CommandListener;)V",
                        (listener.clone(),),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "setTimeout", "(I)V", (50,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "setCurrent",
                        "(Ljavax/microedition/lcdui/Alert;Ljavax/microedition/lcdui/Displayable;)V",
                        (alert.clone(), previous.clone()),
                    )
                    .await?;
                for (now, count) in [(49, 0), (50, 1), (50, 1), (1000, 1)] {
                    clock.set(now);
                    pump_backend_queue(&jvm, &system).await?;
                    assert_eq!(jvm.get_field::<i32>(&listener, "count", "I").await?, count);
                    assert_eq!(current(&jvm, &display).await?.identity(), alert.identity());
                }
                let delivered: ClassInstanceRef<Command> = jvm.get_field(&listener, "lastCommand", "Ljavax/microedition/lcdui/Command;").await?;
                let target: ClassInstanceRef<Displayable> = jvm
                    .get_field(&listener, "lastDisplayable", "Ljavax/microedition/lcdui/Displayable;")
                    .await?;
                assert_eq!(delivered.identity(), command.identity());
                assert_eq!(target.identity(), alert.identity());

                let _: () = jvm
                    .invoke_virtual(
                        &alert,
                        "javax/microedition/lcdui/Alert",
                        "setCommandListener",
                        "(Ljavax/microedition/lcdui/CommandListener;)V",
                        (ClassInstanceRef::<CommandListener>::new(None),),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "handleKeyEvent",
                        "(II)V",
                        (KeyboardEventType::KeyPressed as i32, MIDPKeyCode::LEFT_SOFT_KEY as i32),
                    )
                    .await?;
                assert_eq!(current(&jvm, &display).await?.identity(), previous.identity());
                assert_eq!(jvm.get_field::<i32>(&listener, "count", "I").await?, 1);
                Ok(())
            },
        )
    }

    #[test]
    fn timed_alert_without_previous_screen_clears_the_backing_image() -> Result<()> {
        let clock = TestClock::new();
        run_jvm_test_with_system(
            test_protos(),
            Box::new(TestPlatform::with_clock(clock.clone())),
            move |jvm, system| async move {
                let display = test_midlet_display(&jvm).await?;
                let alert = new_alert(&jvm, "temporary").await?;
                let _: () = jvm
                    .invoke_virtual(&alert, "javax/microedition/lcdui/Alert", "setTimeout", "(I)V", (25,))
                    .await?;
                show(&jvm, &display, JavaValue::from(alert.clone()).into()).await?;
                let mut graphics = Display::screen_graphics(&jvm, &display).await?;
                let screen_image = Graphics::image(&jvm, &mut graphics).await?;
                for expired in [false, true] {
                    if expired {
                        clock.set(25);
                        pump_backend_queue(&jvm, &system).await?;
                        assert!(current(&jvm, &display).await?.is_null());
                        let shown: bool = jvm
                            .invoke_virtual(&alert, "javax/microedition/lcdui/Displayable", "isShown", "()Z", ())
                            .await?;
                        assert!(!shown);
                    }
                    let _: () = jvm
                        .invoke_virtual(&display, "javax/microedition/lcdui/Display", "handlePaintEvent", "()V", ())
                        .await?;
                    let image = Image::image(&jvm, &screen_image).await?;
                    let blank = (0..240).all(|y| {
                        (0..320).all(|x| {
                            let color = image.get_pixel(x, y);
                            (color.r, color.g, color.b) == (0xff, 0xff, 0xff)
                        })
                    });
                    assert_eq!(blank, expired);
                }
                Ok(())
            },
        )
    }
}
