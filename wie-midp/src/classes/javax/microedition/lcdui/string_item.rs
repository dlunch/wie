use alloc::{string::String as RustString, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Font, Graphics, Item};

const LEFT_TOP: i32 = 4 | 16;

// class javax.microedition.lcdui.StringItem
pub struct StringItem;

impl StringItem {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/StringItem",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;I)V",
                    Self::init_with_appearance,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getText", "()Ljava/lang/String;", Self::get_text, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setText", "(Ljava/lang/String;)V", Self::set_text, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getAppearanceMode", "()I", Self::get_appearance_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFont", "(Ljavax/microedition/lcdui/Font;)V", Self::set_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getFont", "()Ljavax/microedition/lcdui/Font;", Self::get_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPreferredSize", "(II)V", Self::set_preferred_size, MethodAccessFlags::PUBLIC),
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
            fields: vec![
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("appearanceMode", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("font", "Ljavax/microedition/lcdui/Font;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/StringItem",
            "<init>",
            "(Ljava/lang/String;Ljava/lang/String;I)V",
            (label, text, 0),
        )
        .await
    }

    async fn init_with_appearance(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        appearance_mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.StringItem::<init>({this:?}, {label:?}, {text:?}, {appearance_mode})");

        if !(0..=2).contains(&appearance_mode) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid appearance mode").await);
        }

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "label", "Ljava/lang/String;", label).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "appearanceMode", "I", appearance_mode).await?;
        jvm.put_field(&mut this, "font", "Ljavax/microedition/lcdui/Font;", ClassInstanceRef::<Font>::new(None))
            .await?;

        Ok(())
    }

    async fn get_text(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "text", "Ljava/lang/String;").await
    }

    async fn set_text(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        Item::invalidate(jvm, &this, true).await
    }

    async fn get_appearance_mode(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "appearanceMode", "I").await
    }

    async fn set_font(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        jvm.put_field(&mut this, "font", "Ljavax/microedition/lcdui/Font;", font).await?;
        Item::invalidate(jvm, &this, true).await
    }

    async fn get_font(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Font>> {
        let font: ClassInstanceRef<Font> = jvm.get_field(&this, "font", "Ljavax/microedition/lcdui/Font;").await?;
        if font.is_null() {
            jvm.invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
                .await
        } else {
            Ok(font)
        }
    }

    async fn set_preferred_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "setPreferredSize", "(II)V", (width, height))
            .await
    }

    async fn minimum_content_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let Some(text) = Self::text(jvm, &this).await? else {
            return Ok(0);
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        let text_width = if appearance_mode == 2 {
            Font::preferred_width(context.system().platform().font(), &text)
        } else {
            Font::minimum_width(context.system().platform().font(), &text)
        };
        Ok(text_width + Item::appearance_inset(appearance_mode) * 2)
    }

    async fn minimum_content_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let Some(text) = Self::text(jvm, &this).await? else {
            return Ok(0);
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        Ok(Font::wrap(context.system().platform().font(), &text, None).len() as i32 * Font::HEIGHT + Item::appearance_inset(appearance_mode) * 2)
    }

    async fn preferred_content_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let Some(text) = Self::text(jvm, &this).await? else {
            return Ok(0);
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        Ok(Font::preferred_width(context.system().platform().font(), &text) + Item::appearance_inset(appearance_mode) * 2)
    }

    async fn preferred_content_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        let Some(text) = Self::text(jvm, &this).await? else {
            return Ok(0);
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        let inset = Item::appearance_inset(appearance_mode);
        let wrap_width = if appearance_mode == 2 || width < 0 {
            None
        } else {
            Some((width - inset * 2).max(1))
        };
        Ok(Font::wrap(context.system().platform().font(), &text, wrap_width).len() as i32 * Font::HEIGHT + inset * 2)
    }

    #[allow(clippy::too_many_arguments)]
    async fn paint_content(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        _focused: bool,
    ) -> JvmResult<()> {
        let Some(text) = Self::text(jvm, &this).await? else {
            return Ok(());
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        Item::paint_appearance(jvm, &graphics, x, y, width, height, appearance_mode).await?;

        let inset = Item::appearance_inset(appearance_mode);
        let content_x = x + inset;
        let content_y = y + inset;
        let content_width = (width - inset * 2).max(0);
        let content_height = (height - inset * 2).max(0);
        if content_width == 0 || content_height == 0 {
            return Ok(());
        }

        let font: ClassInstanceRef<Font> = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/StringItem",
                "getFont",
                "()Ljavax/microedition/lcdui/Font;",
                (),
            )
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
        let color = if appearance_mode == 1 { Item::LINK_COLOR } else { Item::TEXT_COLOR };
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (color,))
            .await?;
        let wrap_width = if appearance_mode == 2 { None } else { Some(content_width.max(1)) };
        for (index, line) in Font::wrap(context.system().platform().font(), &text, wrap_width).iter().enumerate() {
            let line_y = content_y + index as i32 * Font::HEIGHT;
            if line_y >= content_y + content_height {
                break;
            }
            let java_line = JavaLangString::from_rust_string(jvm, line).await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawString",
                    "(Ljava/lang/String;III)V",
                    (java_line, content_x, line_y, LEFT_TOP),
                )
                .await?;
            if appearance_mode == 1 && !line.is_empty() {
                let line_width = Font::text_width(context.system().platform().font(), line).min(content_width);
                let underline_y = (line_y + Font::HEIGHT - 1).min(content_y + content_height - 1);
                let _: () = jvm
                    .invoke_virtual(
                        &graphics,
                        "javax/microedition/lcdui/Graphics",
                        "drawLine",
                        "(IIII)V",
                        (content_x, underline_y, content_x + line_width.saturating_sub(1), underline_y),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn is_focusable(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "isFocusable", "()Z", ()).await
    }

    async fn handle_item_key(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _key: i32) -> JvmResult<i32> {
        Ok(0)
    }

    async fn text(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Option<RustString>> {
        let text: ClassInstanceRef<String> = jvm.get_field(this, "text", "Ljava/lang/String;").await?;
        if text.is_null() {
            return Ok(None);
        }

        let text = JavaLangString::to_rust_string(jvm, &text).await?;
        Ok((!text.is_empty()).then_some(text))
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{
        classes::javax::microedition::lcdui::{Font, Graphics, Image, StringItem},
        get_protos,
    };

    async fn create_image_and_graphics(jvm: &Jvm, width: i32, height: i32) -> JvmResult<(ClassInstanceRef<Image>, ClassInstanceRef<Graphics>)> {
        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
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
        Ok((image, graphics))
    }

    async fn fill(jvm: &Jvm, graphics: &ClassInstanceRef<Graphics>, color: i32, width: i32, height: i32) -> JvmResult<()> {
        let _: () = jvm
            .invoke_virtual(graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (color,))
            .await?;
        jvm.invoke_virtual(
            graphics,
            "javax/microedition/lcdui/Graphics",
            "fillRect",
            "(IIII)V",
            (0, 0, width, height),
        )
        .await
    }

    async fn paint_item<T>(
        jvm: &Jvm,
        item: &ClassInstanceRef<T>,
        graphics: ClassInstanceRef<Graphics>,
        bounds: (i32, i32, i32, i32),
        focused: bool,
    ) -> JvmResult<()> {
        let (x, y, width, height) = bounds;
        jvm.invoke_virtual(
            item,
            "javax/microedition/lcdui/Item",
            "paintItem",
            "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
            (graphics, x, y, width, height, focused),
        )
        .await
    }

    #[test]
    fn string_item_renders_newlines_wrapping_appearance_focus_and_requested_font() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let text = ClassInstanceRef::<String>::from(JavaLangString::from_rust_string(&jvm, "first line\nsecond line wraps here").await?);
            let hyperlink: ClassInstanceRef<StringItem> = jvm
                .new_class(
                    "javax/microedition/lcdui/StringItem",
                    "(Ljava/lang/String;Ljava/lang/String;I)V",
                    (ClassInstanceRef::<String>::new(None), text, 1),
                )
                .await?
                .into();
            let custom_font: ClassInstanceRef<Font> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Font",
                    "getFont",
                    "(III)Ljavax/microedition/lcdui/Font;",
                    (0, 1, 8),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &hyperlink,
                    "javax/microedition/lcdui/StringItem",
                    "setFont",
                    "(Ljavax/microedition/lcdui/Font;)V",
                    (custom_font.clone(),),
                )
                .await?;
            let wrapped_height: i32 = jvm
                .invoke_virtual(&hyperlink, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (54,))
                .await?;
            assert!(wrapped_height >= 36, "explicit newline and width wrapping must both be preserved");

            let (hyperlink_image, hyperlink_graphics) = create_image_and_graphics(&jvm, 80, 70).await?;
            fill(&jvm, &hyperlink_graphics, 0xffffff, 80, 70).await?;
            paint_item(&jvm, &hyperlink, hyperlink_graphics.clone(), (4, 4, 54, wrapped_height), true).await?;
            let active_font: ClassInstanceRef<Font> = jvm
                .invoke_virtual(
                    &hyperlink_graphics,
                    "javax/microedition/lcdui/Graphics",
                    "getFont",
                    "()Ljavax/microedition/lcdui/Font;",
                    (),
                )
                .await?;
            assert_eq!(
                active_font.identity(),
                custom_font.identity(),
                "StringItem paint must select its requested font"
            );

            let hyperlink_pixels = Image::image(&jvm, &hyperlink_image).await?;
            let blue_pixels = (4..58)
                .flat_map(|x| (4..(4 + wrapped_height).min(70)).map(move |y| (x, y)))
                .filter(|&(x, y)| {
                    let color = hyperlink_pixels.get_pixel(x, y);
                    color.b > color.r && color.b > color.g
                })
                .count();
            assert!(blue_pixels > 8, "hyperlink text and underline must be visibly styled");
            let focus_corner = hyperlink_pixels.get_pixel(4, 4);
            assert_eq!((focus_corner.r, focus_corner.g, focus_corner.b), (0x2f, 0x6f, 0x9f));

            let button_text = ClassInstanceRef::<String>::from(JavaLangString::from_rust_string(&jvm, "button text that does not wrap").await?);
            let button: ClassInstanceRef<StringItem> = jvm
                .new_class(
                    "javax/microedition/lcdui/StringItem",
                    "(Ljava/lang/String;Ljava/lang/String;I)V",
                    (ClassInstanceRef::<String>::new(None), button_text, 2),
                )
                .await?
                .into();
            let button_narrow_height: i32 = jvm
                .invoke_virtual(&button, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (35,))
                .await?;
            let button_wide_height: i32 = jvm
                .invoke_virtual(&button, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (180,))
                .await?;
            assert_eq!(button_narrow_height, button_wide_height, "button appearance must not width-wrap text");

            let (button_image, button_graphics) = create_image_and_graphics(&jvm, 90, 30).await?;
            fill(&jvm, &button_graphics, 0xffffff, 90, 30).await?;
            paint_item(&jvm, &button, button_graphics, (5, 5, 70, button_narrow_height), false).await?;
            let button_pixels = Image::image(&jvm, &button_image).await?;
            let border = button_pixels.get_pixel(5, 5);
            assert_ne!(
                (border.r, border.g, border.b),
                (0xff, 0xff, 0xff),
                "button appearance must render a border"
            );

            Ok(())
        })
    }
}
