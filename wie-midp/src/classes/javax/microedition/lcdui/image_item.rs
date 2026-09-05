use alloc::{string::String as RustString, vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::text_layout::{minimum_width, wrap};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Font, Graphics, Image, Item};

const LEFT_TOP: i32 = 4 | 16;

// class javax.microedition.lcdui.ImageItem
pub struct ImageItem;

impl ImageItem {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/ImageItem",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;ILjava/lang/String;)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;ILjava/lang/String;I)V",
                    Self::init_with_appearance,
                    MethodAccessFlags::PUBLIC,
                ),
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
                JavaMethodProto::new("getAltText", "()Ljava/lang/String;", Self::get_alt_text, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setAltText", "(Ljava/lang/String;)V", Self::set_alt_text, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getLayout", "()I", Self::get_layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setLayout", "(I)V", Self::set_layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getAppearanceMode", "()I", Self::get_appearance_mode, MethodAccessFlags::PUBLIC),
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
                JavaFieldProto::new(
                    "LAYOUT_DEFAULT",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "LAYOUT_LEFT",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "LAYOUT_RIGHT",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "LAYOUT_CENTER",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "LAYOUT_NEWLINE_BEFORE",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "LAYOUT_NEWLINE_AFTER",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("image", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("displayImage", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("altText", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("appearanceMode", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        jvm.put_static_field("javax/microedition/lcdui/ImageItem", "LAYOUT_DEFAULT", "I", 0)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/ImageItem", "LAYOUT_LEFT", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/ImageItem", "LAYOUT_RIGHT", "I", 2).await?;
        jvm.put_static_field("javax/microedition/lcdui/ImageItem", "LAYOUT_CENTER", "I", 3)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/ImageItem", "LAYOUT_NEWLINE_BEFORE", "I", 0x100)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/ImageItem", "LAYOUT_NEWLINE_AFTER", "I", 0x200)
            .await?;

        Ok(())
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
        layout: i32,
        alt_text: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/ImageItem",
            "<init>",
            "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;ILjava/lang/String;I)V",
            (label, image, layout, alt_text, 0),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_with_appearance(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
        layout: i32,
        alt_text: ClassInstanceRef<String>,
        appearance_mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.ImageItem::<init>({this:?}, {label:?}, {image:?}, {layout}, {alt_text:?}, {appearance_mode})");

        if layout & !0x7f33 != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid ImageItem layout").await);
        }
        if !(0..=2).contains(&appearance_mode) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid appearance mode").await);
        }

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await?;
        let display_image = Self::snapshot(jvm, &image).await?;
        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Item", "setLabel", "(Ljava/lang/String;)V", (label,))
            .await?;
        let _: () = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Item", "setLayout", "(I)V", (layout,))
            .await?;
        jvm.put_field(&mut this, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(&mut this, "displayImage", "Ljavax/microedition/lcdui/Image;", display_image)
            .await?;
        jvm.put_field(&mut this, "altText", "Ljava/lang/String;", alt_text).await?;
        jvm.put_field(&mut this, "appearanceMode", "I", appearance_mode).await?;

        Ok(())
    }

    async fn snapshot(jvm: &Jvm, image: &ClassInstanceRef<Image>) -> JvmResult<ClassInstanceRef<Image>> {
        if image.is_null() {
            Ok(None.into())
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

    async fn get_image(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Image>> {
        jvm.get_field(&this, "image", "Ljavax/microedition/lcdui/Image;").await
    }

    async fn set_image(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        let display_image = Self::snapshot(jvm, &image).await?;
        jvm.put_field(&mut this, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(&mut this, "displayImage", "Ljavax/microedition/lcdui/Image;", display_image)
            .await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn get_alt_text(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        jvm.get_field(&this, "altText", "Ljava/lang/String;").await
    }

    async fn set_alt_text(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        alt_text: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        jvm.put_field(&mut this, "altText", "Ljava/lang/String;", alt_text).await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Item", "invalidate", "(Z)V", (true,))
            .await
    }

    async fn get_layout(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "getLayout", "()I", ()).await
    }

    async fn set_layout(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, layout: i32) -> JvmResult<()> {
        if layout & !0x7f33 != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid ImageItem layout").await);
        }

        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "setLayout", "(I)V", (layout,))
            .await
    }

    async fn get_appearance_mode(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "appearanceMode", "I").await
    }

    async fn minimum_content_width(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let Some((_image, image_width, _image_height)) = Self::display_image(jvm, &this).await? else {
            return Ok(0);
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        let alt_width = Self::alt_text(jvm, &this)
            .await?
            .as_deref()
            .map(|text| minimum_width(context.system().platform().font(), text, 10.0))
            .filter(|width| *width > 0);
        Ok(image_width.min(alt_width.unwrap_or(image_width)) + Item::appearance_inset(appearance_mode) * 2)
    }

    async fn minimum_content_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let Some((_image, _image_width, image_height)) = Self::display_image(jvm, &this).await? else {
            return Ok(0);
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        let alt_height = Self::alt_text(jvm, &this)
            .await?
            .map(|alt_text| wrap(context.system().platform().font(), &alt_text, 10.0, None).len() as i32 * Font::HEIGHT)
            .filter(|height| *height > 0);
        Ok(image_height.min(alt_height.unwrap_or(image_height)) + Item::appearance_inset(appearance_mode) * 2)
    }

    async fn preferred_content_width(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let Some((_image, image_width, _image_height)) = Self::display_image(jvm, &this).await? else {
            return Ok(0);
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        Ok(image_width + Item::appearance_inset(appearance_mode) * 2)
    }

    async fn preferred_content_height(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        let Some((_image, image_width, image_height)) = Self::display_image(jvm, &this).await? else {
            return Ok(0);
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        let inset = Item::appearance_inset(appearance_mode);
        let available_width = if width < 0 { image_width } else { (width - inset * 2).max(0) };
        if image_width <= available_width {
            return Ok(image_height + inset * 2);
        }

        let Some(alt_text) = Self::alt_text(jvm, &this).await? else {
            return Ok(image_height + inset * 2);
        };
        let wrap_width = if appearance_mode == 2 { None } else { Some(available_width.max(1)) };
        Ok(wrap(context.system().platform().font(), &alt_text, 10.0, wrap_width).len() as i32 * Font::HEIGHT + inset * 2)
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
        let Some((image, image_width, image_height)) = Self::display_image(jvm, &this).await? else {
            return Ok(());
        };
        let appearance_mode: i32 = jvm.get_field(&this, "appearanceMode", "I").await?;
        let _: () = jvm
            .invoke_static(
                "javax/microedition/lcdui/Item",
                "paintAppearance",
                "(Ljavax/microedition/lcdui/Graphics;IIIII)V",
                (graphics.clone(), x, y, width, height, appearance_mode),
            )
            .await?;

        let inset = Item::appearance_inset(appearance_mode);
        let content_x = x + inset;
        let content_y = y + inset;
        let content_width = (width - inset * 2).max(0);
        let content_height = (height - inset * 2).max(0);
        if content_width == 0 || content_height == 0 {
            return Ok(());
        }

        let alt_text = Self::alt_text(jvm, &this).await?;
        if (image_width > content_width || image_height > content_height) && alt_text.is_some() {
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
            let color = if appearance_mode == 1 { Item::LINK_COLOR } else { Item::TEXT_COLOR };
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (color,))
                .await?;
            let wrap_width = if appearance_mode == 2 { None } else { Some(content_width.max(1)) };
            for (index, line) in wrap(
                context.system().platform().font(),
                alt_text.as_deref().unwrap_or_default(),
                10.0,
                wrap_width,
            )
            .iter()
            .enumerate()
            {
                let line_y = content_y + index as i32 * Font::HEIGHT;
                if line_y >= content_y + content_height {
                    break;
                }
                let line = JavaLangString::from_rust_string(jvm, line).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &graphics,
                        "javax/microedition/lcdui/Graphics",
                        "drawString",
                        "(Ljava/lang/String;III)V",
                        (line, content_x, line_y, LEFT_TOP),
                    )
                    .await?;
            }
            return Ok(());
        }

        let layout: i32 = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "getLayout", "()I", ()).await?;
        let image_x = match layout & 0x3 {
            2 => content_x + content_width - image_width,
            3 => content_x + (content_width - image_width) / 2,
            _ => content_x,
        };
        let image_y = match layout & 0x30 {
            0x20 => content_y + content_height - image_height,
            0x30 => content_y + (content_height - image_height) / 2,
            _ => content_y,
        };
        jvm.invoke_virtual(
            &graphics,
            "javax/microedition/lcdui/Graphics",
            "drawImage",
            "(Ljavax/microedition/lcdui/Image;III)V",
            (image, image_x, image_y, LEFT_TOP),
        )
        .await
    }

    async fn is_focusable(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "isFocusable", "()Z", ()).await
    }

    async fn handle_item_key(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>, _key: i32) -> JvmResult<i32> {
        Ok(0)
    }

    async fn display_image(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Option<(ClassInstanceRef<Image>, i32, i32)>> {
        let image: ClassInstanceRef<Image> = jvm.get_field(this, "displayImage", "Ljavax/microedition/lcdui/Image;").await?;
        if image.is_null() {
            return Ok(None);
        }

        let width: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
            .await?;
        Ok(Some((image, width, height)))
    }

    async fn alt_text(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Option<RustString>> {
        let alt_text: ClassInstanceRef<String> = jvm.get_field(this, "altText", "Ljava/lang/String;").await?;
        if alt_text.is_null() {
            return Ok(None);
        }

        let alt_text = JavaLangString::to_rust_string(jvm, &alt_text).await?;
        Ok((!alt_text.is_empty()).then_some(alt_text))
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
        classes::javax::microedition::lcdui::{Graphics, Image, ImageItem},
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
    fn image_item_uses_snapshots_alt_text_and_horizontal_layout() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (source, source_graphics) = create_image_and_graphics(&jvm, 8, 4).await?;
            fill(&jvm, &source_graphics, 0xcc1122, 8, 4).await?;
            let alt_text = ClassInstanceRef::<String>::from(JavaLangString::from_rust_string(&jvm, "image unavailable").await?);
            let item: ClassInstanceRef<ImageItem> = jvm
                .new_class(
                    "javax/microedition/lcdui/ImageItem",
                    "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;ILjava/lang/String;)V",
                    (None, source.clone(), 2, alt_text.clone()),
                )
                .await?
                .into();
            let returned_source: ClassInstanceRef<Image> = jvm
                .invoke_virtual(
                    &item,
                    "javax/microedition/lcdui/ImageItem",
                    "getImage",
                    "()Ljavax/microedition/lcdui/Image;",
                    (),
                )
                .await?;
            assert_eq!(returned_source.identity(), source.identity());

            fill(&jvm, &source_graphics, 0x2255dd, 8, 4).await?;
            let (first_target, first_graphics) = create_image_and_graphics(&jvm, 30, 12).await?;
            fill(&jvm, &first_graphics, 0xffffff, 30, 12).await?;
            paint_item(&jvm, &item, first_graphics, (0, 0, 30, 12), false).await?;
            let first_pixels = Image::image(&jvm, &first_target).await?;
            let snapshot_pixel = first_pixels.get_pixel(22, 0);
            assert_eq!((snapshot_pixel.r, snapshot_pixel.g, snapshot_pixel.b), (0xcc, 0x11, 0x22));
            let left_pixel = first_pixels.get_pixel(0, 0);
            assert_eq!(
                (left_pixel.r, left_pixel.g, left_pixel.b),
                (0xff, 0xff, 0xff),
                "LAYOUT_RIGHT must align the image to the right"
            );

            let _: () = jvm
                .invoke_virtual(
                    &item,
                    "javax/microedition/lcdui/ImageItem",
                    "setImage",
                    "(Ljavax/microedition/lcdui/Image;)V",
                    (source.clone(),),
                )
                .await?;
            let (second_target, second_graphics) = create_image_and_graphics(&jvm, 30, 12).await?;
            fill(&jvm, &second_graphics, 0xffffff, 30, 12).await?;
            paint_item(&jvm, &item, second_graphics, (0, 0, 30, 12), false).await?;
            let second_pixels = Image::image(&jvm, &second_target).await?;
            let refreshed_pixel = second_pixels.get_pixel(22, 0);
            assert_eq!((refreshed_pixel.r, refreshed_pixel.g, refreshed_pixel.b), (0x22, 0x55, 0xdd));

            let (oversized_source, oversized_graphics) = create_image_and_graphics(&jvm, 80, 6).await?;
            fill(&jvm, &oversized_graphics, 0xcc1122, 80, 6).await?;
            let oversized: ClassInstanceRef<ImageItem> = jvm
                .new_class(
                    "javax/microedition/lcdui/ImageItem",
                    "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;ILjava/lang/String;)V",
                    (None, oversized_source, 1, alt_text),
                )
                .await?
                .into();
            let alt_height: i32 = jvm
                .invoke_virtual(&oversized, "javax/microedition/lcdui/Item", "measureHeight", "(I)I", (35,))
                .await?;
            assert!(alt_height >= 24, "alternate text must wrap when the image cannot fit");
            let (alt_target, alt_graphics) = create_image_and_graphics(&jvm, 35, alt_height).await?;
            fill(&jvm, &alt_graphics, 0xffffff, 35, alt_height).await?;
            paint_item(&jvm, &oversized, alt_graphics, (0, 0, 35, alt_height), false).await?;
            let alt_pixels = Image::image(&jvm, &alt_target).await?;
            let red_pixels = (0..35)
                .flat_map(|x| (0..alt_height).map(move |y| (x, y)))
                .filter(|&(x, y)| {
                    let color = alt_pixels.get_pixel(x, y);
                    color.r == 0xcc && color.g == 0x11 && color.b == 0x22
                })
                .count();
            let non_white_pixels = (0..35)
                .flat_map(|x| (0..alt_height).map(move |y| (x, y)))
                .filter(|&(x, y)| {
                    let color = alt_pixels.get_pixel(x, y);
                    (color.r, color.g, color.b) != (0xff, 0xff, 0xff)
                })
                .count();
            assert_eq!(red_pixels, 0, "oversized image must be replaced by alternate text");
            assert!(non_white_pixels > 8, "alternate text must render visible glyphs");

            Ok(())
        })
    }
}
