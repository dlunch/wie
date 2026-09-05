use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{lang::String, util::Vector};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::{Command, Graphics, Item, ItemCommandListener},
    net::wie::MIDPKeyCode,
};

const GAUGE_MINIMUM_WIDTH: i32 = 24;
const GAUGE_PREFERRED_WIDTH: i32 = 96;
const GAUGE_HEIGHT: i32 = 12;
const TRACK_BACKGROUND: i32 = 0xd2dae1;
const TRACK_BORDER: i32 = 0x596773;
const TRACK_FILL: i32 = 0x2f7eb8;

// class javax.microedition.lcdui.Gauge
pub struct Gauge;

impl Gauge {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Gauge",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;ZII)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxValue", "()I", Self::get_max_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxValue", "(I)V", Self::set_max_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getValue", "()I", Self::get_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setValue", "(I)V", Self::set_value, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isInteractive", "()Z", Self::is_interactive, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setLabel", "(Ljava/lang/String;)V", Self::set_label, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setLayout", "(I)V", Self::set_layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "addCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::add_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setItemCommandListener",
                    "(Ljavax/microedition/lcdui/ItemCommandListener;)V",
                    Self::set_item_command_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setPreferredSize", "(II)V", Self::set_preferred_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setDefaultCommand",
                    "(Ljavax/microedition/lcdui/Command;)V",
                    Self::set_default_command,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("minimumContentWidth", "()I", Self::minimum_content_width, MethodAccessFlags::empty()),
                JavaMethodProto::new("minimumContentHeight", "()I", Self::minimum_content_height, MethodAccessFlags::empty()),
                JavaMethodProto::new("preferredContentWidth", "()I", Self::preferred_content_width, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "preferredContentHeight",
                    "(I)I",
                    Self::preferred_content_height,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new(
                    "paintContent",
                    "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                    Self::paint_content,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("isFocusable", "()Z", Self::is_focusable, MethodAccessFlags::empty()),
                JavaMethodProto::new("handleItemKey", "(I)I", Self::handle_item_key, MethodAccessFlags::empty()),
            ],
            fields: [
                "INDEFINITE",
                "CONTINUOUS_IDLE",
                "INCREMENTAL_IDLE",
                "CONTINUOUS_RUNNING",
                "INCREMENTAL_UPDATING",
            ]
            .into_iter()
            .map(|name| JavaFieldProto::new(name, "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL))
            .chain([
                JavaFieldProto::new("interactive", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("maxValue", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("value", "I", FieldAccessFlags::PRIVATE),
            ])
            .collect(),
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field("javax/microedition/lcdui/Gauge", "INDEFINITE", "I", -1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Gauge", "CONTINUOUS_IDLE", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Gauge", "INCREMENTAL_IDLE", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Gauge", "CONTINUOUS_RUNNING", "I", 2)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Gauge", "INCREMENTAL_UPDATING", "I", 3)
            .await?;

        Ok(())
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        interactive: bool,
        max_value: i32,
        initial_value: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Gauge::<init>({this:?}, {label:?}, {interactive}, {max_value}, {initial_value})");

        if max_value <= 0 && (interactive || max_value != -1) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid Gauge maximum value").await);
        }
        if max_value == -1 && !(0..=3).contains(&initial_value) {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Invalid indefinite Gauge state")
                .await);
        }

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "label", "Ljava/lang/String;", label).await?;
        jvm.put_field(&mut this, "interactive", "Z", interactive).await?;
        jvm.put_field(&mut this, "maxValue", "I", max_value).await?;
        jvm.put_field(
            &mut this,
            "value",
            "I",
            if max_value == -1 {
                initial_value
            } else {
                initial_value.clamp(0, max_value)
            },
        )
        .await?;

        Ok(())
    }

    async fn get_max_value(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "maxValue", "I").await
    }

    async fn set_max_value(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_value: i32) -> JvmResult<()> {
        let interactive: bool = jvm.get_field(&this, "interactive", "Z").await?;
        if max_value <= 0 && (interactive || max_value != -1) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid Gauge maximum value").await);
        }

        let old_max: i32 = jvm.get_field(&this, "maxValue", "I").await?;
        let old_value: i32 = jvm.get_field(&this, "value", "I").await?;
        let value = match (old_max, max_value) {
            (-1, -1) => old_value,
            (-1, _) => 0,
            (_, -1) => 0,
            (_, _) => old_value.min(max_value),
        };
        jvm.put_field(&mut this, "maxValue", "I", max_value).await?;
        jvm.put_field(&mut this, "value", "I", value).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
            .await
    }

    async fn get_value(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "value", "I").await
    }

    async fn set_value(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        let max_value: i32 = jvm.get_field(&this, "maxValue", "I").await?;
        if max_value == -1 && !(0..=3).contains(&value) {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Invalid indefinite Gauge state")
                .await);
        }

        jvm.put_field(&mut this, "value", "I", if max_value == -1 { value } else { value.clamp(0, max_value) })
            .await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
            .await
    }

    async fn is_interactive(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.get_field(&this, "interactive", "Z").await
    }

    async fn set_label(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, label: ClassInstanceRef<String>) -> JvmResult<()> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "setLabel", "(Ljava/lang/String;)V", (label,))
            .await
    }

    async fn set_layout(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, layout: i32) -> JvmResult<()> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "setLayout", "(I)V", (layout,))
            .await
    }

    async fn add_command(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<()> {
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Item",
            "addCommand",
            "(Ljavax/microedition/lcdui/Command;)V",
            (command,),
        )
        .await
    }

    async fn set_item_command_listener(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ItemCommandListener>,
    ) -> JvmResult<()> {
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Item",
            "setItemCommandListener",
            "(Ljavax/microedition/lcdui/ItemCommandListener;)V",
            (listener,),
        )
        .await
    }

    async fn set_preferred_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "setPreferredSize", "(II)V", (width, height))
            .await
    }

    async fn set_default_command(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Item",
            "setDefaultCommand",
            "(Ljavax/microedition/lcdui/Command;)V",
            (command,),
        )
        .await
    }

    async fn minimum_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(GAUGE_MINIMUM_WIDTH)
    }

    async fn minimum_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(GAUGE_HEIGHT)
    }

    async fn preferred_content_width(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(GAUGE_PREFERRED_WIDTH)
    }

    async fn preferred_content_height(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _width: i32) -> JvmResult<i32> {
        Ok(GAUGE_HEIGHT)
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_content(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        _focused: bool,
    ) -> JvmResult<()> {
        if width <= 0 || height <= 0 {
            return Ok(());
        }

        let track_x = x + 2.min(width);
        let track_width = (width - 4).max(1);
        let track_height = 8.min(height).max(1);
        let track_y = y + (height - track_height) / 2;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (TRACK_BACKGROUND,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "fillRect",
                "(IIII)V",
                (track_x, track_y, track_width, track_height),
            )
            .await?;

        let max_value: i32 = jvm.get_field(&this, "maxValue", "I").await?;
        let value: i32 = jvm.get_field(&this, "value", "I").await?;
        let (fill_x, fill_width) = if max_value > 0 {
            (track_x, track_width.saturating_mul(value) / max_value)
        } else {
            match value {
                1 => (track_x, (track_width / 4).max(1)),
                2 => (track_x + track_width / 3, (track_width / 3).max(1)),
                3 => (track_x, (track_width * 2 / 3).max(1)),
                _ => (track_x, 0),
            }
        };
        if fill_width > 0 {
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (TRACK_FILL,))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "fillRect",
                    "(IIII)V",
                    (fill_x, track_y, fill_width.min(track_width), track_height),
                )
                .await?;
        }

        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (TRACK_BORDER,))
            .await?;
        jvm.invoke_virtual(
            &graphics,
            "javax/microedition/lcdui/Graphics",
            "drawRect",
            "(IIII)V",
            (track_x, track_y, track_width - 1, track_height - 1),
        )
        .await
    }

    async fn is_focusable(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        if jvm.get_field::<bool>(&this, "interactive", "Z").await? {
            return Ok(true);
        }

        let commands: ClassInstanceRef<Vector> = jvm.get_field(&this, "commands", "Ljava/util/Vector;").await?;
        Ok(jvm.invoke_virtual::<_, i32>(&commands, "java/util/Vector", "size", "()I", ()).await? > 0)
    }

    async fn handle_item_key(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: i32) -> JvmResult<i32> {
        let interactive: bool = jvm.get_field(&this, "interactive", "Z").await?;
        if !interactive || (key != MIDPKeyCode::LEFT as i32 && key != MIDPKeyCode::RIGHT as i32) {
            return Ok(0);
        }

        let max_value: i32 = jvm.get_field(&this, "maxValue", "I").await?;
        let value: i32 = jvm.get_field(&this, "value", "I").await?;
        let new_value = if key == MIDPKeyCode::LEFT as i32 {
            value.saturating_sub(1).max(0)
        } else {
            value.saturating_add(1).min(max_value)
        };
        if new_value == value {
            return Ok(Item::INPUT_HANDLED);
        }

        jvm.put_field(&mut this, "value", "I", new_value).await?;
        let _: () = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (false,))
            .await?;
        Ok(Item::INPUT_HANDLED | Item::INPUT_CHANGED)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{
        classes::javax::microedition::lcdui::{Gauge, Graphics, Image, Item},
        get_protos,
    };

    use crate::classes::net::wie::MIDPKeyCode;

    #[test]
    fn gauge_clamps_input_and_renders_value_and_focus() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let gauge: ClassInstanceRef<Gauge> = jvm
                .new_class("javax/microedition/lcdui/Gauge", "(Ljava/lang/String;ZII)V", (None, true, 2, 1))
                .await?
                .into();
            for (key, result, value) in [
                (MIDPKeyCode::LEFT, Item::INPUT_HANDLED | Item::INPUT_CHANGED, 0),
                (MIDPKeyCode::LEFT, Item::INPUT_HANDLED, 0),
                (MIDPKeyCode::RIGHT, Item::INPUT_HANDLED | Item::INPUT_CHANGED, 1),
            ] {
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&gauge, "javax/microedition/lcdui/Item", "handleItemKey", "(I)I", (key as i32,))
                        .await?,
                    result
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&gauge, "javax/microedition/lcdui/Gauge", "getValue", "()I", ())
                        .await?,
                    value
                );
            }
            let invalid: JvmResult<()> = jvm
                .invoke_virtual(&gauge, "javax/microedition/lcdui/Gauge", "setMaxValue", "(I)V", (-1,))
                .await;
            let Err(JavaError::JavaException(exception)) = invalid else {
                panic!("interactive Gauge accepted INDEFINITE");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (90, 30),
                )
                .await?;
            let graphics: ClassInstanceRef<Graphics> = jvm
                .invoke_virtual(
                    &image,
                    "javax/microedition/lcdui/Image",
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    (),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &gauge,
                    "javax/microedition/lcdui/Item",
                    "paintItem",
                    "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
                    (graphics, 4, 4, 80, 20, true),
                )
                .await?;
            let pixels = Image::image(&jvm, &image).await?;
            let focus = pixels.get_pixel(4, 4);
            assert_eq!((focus.r, focus.g, focus.b), (0x2f, 0x6f, 0x9f));
            let fill = pixels.get_pixel(10, 14);
            let track = pixels.get_pixel(78, 14);
            assert_ne!((fill.r, fill.g, fill.b), (track.r, track.g, track.b));

            let indicator: ClassInstanceRef<Gauge> = jvm
                .new_class("javax/microedition/lcdui/Gauge", "(Ljava/lang/String;ZII)V", (None, false, 10, 7))
                .await?
                .into();
            for (maximum, expected) in [(5, 5), (-1, 0), (10, 0)] {
                let _: () = jvm
                    .invoke_virtual(&indicator, "javax/microedition/lcdui/Gauge", "setMaxValue", "(I)V", (maximum,))
                    .await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&indicator, "javax/microedition/lcdui/Gauge", "getValue", "()I", ())
                        .await?,
                    expected
                );
            }
            Ok(())
        })
    }
}
