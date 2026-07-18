use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_backend::canvas::{PixelType, Rgb8Pixel};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

// class com.skt.m.Graphics2D
pub struct Graphics2D;

impl Graphics2D {
    const DRAW_COPY: i32 = 0;
    const DRAW_AND: i32 = 1;
    const DRAW_OR: i32 = 2;
    const DRAW_XOR: i32 = 3;

    pub fn as_proto() -> WieJavaClassProto {
        let public = MethodAccessFlags::PUBLIC;
        let public_static = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;
        let public_static_field = FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL;

        WieJavaClassProto {
            name: "com/skt/m/Graphics2D",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Graphics;)V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new(
                    "getGraphics2D",
                    "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                    Self::get_graphics2d,
                    public_static,
                ),
                JavaMethodProto::new("captureLCD", "(IIII)Ljavax/microedition/lcdui/Image;", Self::capture_lcd, public_static),
                JavaMethodProto::new("drawImage", "(IILjavax/microedition/lcdui/Image;IIIII)V", Self::draw_image, public),
                JavaMethodProto::new(
                    "createMaskableImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    Self::create_maskable_image,
                    public_static,
                ),
                JavaMethodProto::new("getPixel", "(II)I", Self::get_pixel, public),
                JavaMethodProto::new("getPixelMask", "(II)Z", Self::get_pixel_mask, public),
                JavaMethodProto::new("invertRect", "(IIII)V", Self::invert_rect, public),
                JavaMethodProto::new("setPixel", "(III)V", Self::set_pixel, public),
                JavaMethodProto::new("setPixelMask", "(IIZ)V", Self::set_pixel_mask, public),
            ],
            fields: vec![
                JavaFieldProto::new("DRAW_COPY", "I", public_static_field),
                JavaFieldProto::new("DRAW_AND", "I", public_static_field),
                JavaFieldProto::new("DRAW_OR", "I", public_static_field),
                JavaFieldProto::new("DRAW_XOR", "I", public_static_field),
                JavaFieldProto::new("graphics", "Ljavax/microedition/lcdui/Graphics;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::<clinit>()");

        for (name, value) in [
            ("DRAW_COPY", Self::DRAW_COPY),
            ("DRAW_AND", Self::DRAW_AND),
            ("DRAW_OR", Self::DRAW_OR),
            ("DRAW_XOR", Self::DRAW_XOR),
        ] {
            jvm.put_static_field("com/skt/m/Graphics2D", name, "I", value).await?;
        }

        Ok(())
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::<init>({this:?}, {graphics:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        jvm.put_field(&mut this, "graphics", "Ljavax/microedition/lcdui/Graphics;", graphics)
            .await?;

        Ok(())
    }

    async fn get_graphics2d(jvm: &Jvm, _context: &mut WieJvmContext, graphics: ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("com.skt.m.Graphics2D::getGraphics2D({graphics:?})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let instance = jvm
            .new_class("com/skt/m/Graphics2D", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics,))
            .await?;

        Ok(instance.into())
    }

    async fn capture_lcd(jvm: &Jvm, _context: &mut WieJvmContext, x: i32, y: i32, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub com.skt.m.Graphics2D::captureLCD({x}, {y}, {width}, {height})");

        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must be positive")
                .await);
        }

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        Ok(image)
    }

    #[allow(clippy::too_many_arguments)]
    async fn draw_image(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        tx: i32,
        ty: i32,
        src: ClassInstanceRef<Image>,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::drawImage({this:?}, {tx}, {ty}, {src:?}, {sx}, {sy}, {sw}, {sh}, {mode})");

        if src.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

        if ![Self::DRAW_COPY, Self::DRAW_AND, Self::DRAW_OR, Self::DRAW_XOR].contains(&mode) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid draw mode").await);
        }

        if sw < 0 || sh < 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must not be negative")
                .await);
        }

        if sw == 0 || sh == 0 {
            return Ok(());
        }

        if mode == Self::DRAW_AND || mode == Self::DRAW_OR {
            tracing::warn!("stub com.skt.m.Graphics2D::drawImage unsupported mode {mode}");
            return Ok(());
        }

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let clip = Graphics::clip(jvm, &graphics).await?;
        let src_image = Image::image(jvm, &src).await?;

        let image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        canvas.set_xor_mode(mode == Self::DRAW_XOR);

        canvas.draw(
            tx.wrapping_add(translate_x),
            ty.wrapping_add(translate_y),
            sw as _,
            sh as _,
            &*src_image,
            sx,
            sy,
            clip,
        );

        Ok(())
    }

    async fn create_maskable_image(jvm: &Jvm, _context: &mut WieJvmContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.skt.m.Graphics2D::createMaskableImage({width}, {height})");

        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must be positive")
                .await);
        }

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        Ok(image)
    }

    async fn get_pixel(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<i32> {
        tracing::debug!("com.skt.m.Graphics2D::getPixel({this:?}, {x}, {y})");

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let image = Graphics::image(jvm, &mut graphics).await?;
        let canvas = Image::canvas(jvm, &image).await?;

        Ok(canvas
            .get_pixel(x.wrapping_add(translate_x), y.wrapping_add(translate_y))
            .map(|color| ((color.r as i32) << 16) | ((color.g as i32) << 8) | color.b as i32)
            .unwrap_or(0))
    }

    async fn get_pixel_mask(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Graphics2D::getPixelMask({this:?}, {x}, {y})");
        Ok(false)
    }

    async fn invert_rect(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::invertRect({this:?}, {x}, {y}, {width}, {height})");

        if width < 0 || height < 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must not be negative")
                .await);
        }

        if width == 0 || height == 0 {
            return Ok(());
        }

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let clip = Graphics::clip(jvm, &graphics).await?;
        let image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        canvas.invert_rect(
            x.wrapping_add(translate_x),
            y.wrapping_add(translate_y),
            width as u32,
            height as u32,
            clip,
        );

        Ok(())
    }

    async fn set_pixel(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, color: i32) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::setPixel({this:?}, {x}, {y}, {color})");

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let clip = Graphics::clip(jvm, &graphics).await?;
        let image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        canvas.set_pixel(
            x.wrapping_add(translate_x),
            y.wrapping_add(translate_y),
            Rgb8Pixel::to_color(color as u32),
            clip,
        );

        Ok(())
    }

    async fn set_pixel_mask(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, mask: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Graphics2D::setPixelMask({this:?}, {x}, {y}, {mask})");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, JavaError, Result as JvmResult};
    use test_utils::run_jvm_test;

    use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

    use crate::classes::com::skt::m::SISImage;

    use super::Graphics2D;

    #[test]
    fn graphics_2d_copy_xor_clip_and_pixel_operations_use_the_target_canvas() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [Graphics2D::as_proto()].into()]),
            |jvm| async move {
                let target: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (4, 1),
                    )
                    .await?;
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(&target, "getGraphics", "()Ljavax/microedition/lcdui/Graphics;", ())
                    .await?;
                let graphics_2d: ClassInstanceRef<Graphics2D> = jvm
                    .invoke_static(
                        "com/skt/m/Graphics2D",
                        "getGraphics2D",
                        "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                        (graphics.clone(),),
                    )
                    .await?;

                let _: () = jvm.invoke_virtual(&graphics, "translate", "(II)V", (1, 0)).await?;
                let _: () = jvm.invoke_virtual(&graphics, "setClip", "(IIII)V", (0, 0, 2, 1)).await?;
                let _: () = jvm.invoke_virtual(&graphics_2d, "setPixel", "(III)V", (0, 0, 0x123456)).await?;
                let _: () = jvm.invoke_virtual(&graphics_2d, "setPixel", "(III)V", (2, 0, 0xffffff)).await?;
                let pixel: i32 = jvm.invoke_virtual(&graphics_2d, "getPixel", "(II)I", (0, 0)).await?;
                assert_eq!(pixel, 0x123456);

                let source: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (2, 1),
                    )
                    .await?;
                let source_graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(&source, "getGraphics", "()Ljavax/microedition/lcdui/Graphics;", ())
                    .await?;
                let _: () = jvm.invoke_virtual(&source_graphics, "setColor", "(I)V", (0xf00faa,)).await?;
                let _: () = jvm.invoke_virtual(&source_graphics, "fillRect", "(IIII)V", (0, 0, 1, 1)).await?;
                let _: () = jvm.invoke_virtual(&source_graphics, "setColor", "(I)V", (0xabcdef,)).await?;
                let _: () = jvm.invoke_virtual(&source_graphics, "fillRect", "(IIII)V", (1, 0, 1, 1)).await?;

                let copy_mode: i32 = jvm.get_static_field("com/skt/m/Graphics2D", "DRAW_COPY", "I").await?;
                let xor_mode: i32 = jvm.get_static_field("com/skt/m/Graphics2D", "DRAW_XOR", "I").await?;
                let _: () = jvm
                    .invoke_virtual(
                        &graphics_2d,
                        "drawImage",
                        "(IILjavax/microedition/lcdui/Image;IIIII)V",
                        (0, 0, source.clone(), 0, 0, 2, 1, copy_mode),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &graphics_2d,
                        "drawImage",
                        "(IILjavax/microedition/lcdui/Image;IIIII)V",
                        (0, 0, source, 0, 0, 1, 1, xor_mode),
                    )
                    .await?;

                let target_image = Image::image(&jvm, &target).await?;
                let untouched = target_image.get_pixel(0, 0);
                assert_eq!((untouched.r, untouched.g, untouched.b), (0, 0, 0));
                let xored = target_image.get_pixel(1, 0);
                assert_eq!((xored.r, xored.g, xored.b), (0, 0, 0));
                let copied = target_image.get_pixel(2, 0);
                assert_eq!((copied.r, copied.g, copied.b), (0xab, 0xcd, 0xef));
                let clipped = target_image.get_pixel(3, 0);
                assert_eq!((clipped.r, clipped.g, clipped.b), (0, 0, 0));

                let _: () = jvm.invoke_virtual(&graphics_2d, "invertRect", "(IIII)V", (0, 0, 4, 1)).await?;
                let target_image = Image::image(&jvm, &target).await?;
                let inverted = target_image.get_pixel(1, 0);
                assert_eq!((inverted.r, inverted.g, inverted.b), (0xff, 0xff, 0xff));
                let outside_clip = target_image.get_pixel(3, 0);
                assert_eq!((outside_clip.r, outside_clip.g, outside_clip.b), (0, 0, 0));

                let _: () = jvm.invoke_virtual(&graphics_2d, "invertRect", "(IIII)V", (0, 0, 0, 1)).await?;
                let negative_size: JvmResult<()> = jvm.invoke_virtual(&graphics_2d, "invertRect", "(IIII)V", (0, 0, -1, 1)).await;
                let Err(JavaError::JavaException(exception)) = negative_size else {
                    panic!("Graphics2D.invertRect accepted a negative width");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn sis_image_placeholder_validates_ranges_and_paints_without_panicking() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [SISImage::as_proto()].into()]),
            |jvm| async move {
                let mut data: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 3).await?.into();
                jvm.store_array(&mut data, 0, [1i8, 2, 3]).await?;

                let _: () = jvm.invoke_static("com/skt/m/SISImage", "createBuffer", "(II)V", (1, 1)).await?;
                for sizes in [(0, 1), (1, 0)] {
                    let invalid_size: JvmResult<()> = jvm.invoke_static("com/skt/m/SISImage", "createBuffer", "(II)V", sizes).await;
                    let Err(JavaError::JavaException(exception)) = invalid_size else {
                        panic!("SISImage.createBuffer accepted a zero buffer size");
                    };
                    assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                }

                let required: i32 = jvm
                    .invoke_static("com/skt/m/SISImage", "getRequiredBufferSize", "([BII)I", (data.clone(), 1, 2))
                    .await?;
                assert_eq!(required, 0);

                let sis_image: ClassInstanceRef<SISImage> = jvm
                    .invoke_static("com/skt/m/SISImage", "createSISImage", "([BII)Lcom/skt/m/SISImage;", (data.clone(), 0, 3))
                    .await?;
                let max_frame_id: i32 = jvm.invoke_virtual(&sis_image, "getMaxFrameID", "()I", ()).await?;
                let max_object_id: i32 = jvm.invoke_virtual(&sis_image, "getMaxObjectID", "()I", ()).await?;
                assert_eq!((max_frame_id, max_object_id), (0, 0));

                let frame: ClassInstanceRef<Image> = jvm
                    .invoke_virtual(&sis_image, "getFrame", "(I)Ljavax/microedition/lcdui/Image;", (0,))
                    .await?;
                let object: ClassInstanceRef<Image> = jvm
                    .invoke_virtual(&sis_image, "getObject", "(IZ)Ljavax/microedition/lcdui/Image;", (0, false))
                    .await?;
                assert!(frame.is_null());
                assert!(object.is_null());

                let invalid_frame: JvmResult<ClassInstanceRef<Image>> = jvm
                    .invoke_virtual(&sis_image, "getFrame", "(I)Ljavax/microedition/lcdui/Image;", (1,))
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_frame else {
                    panic!("SISImage.getFrame accepted an out-of-range ID");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let invalid_object: JvmResult<ClassInstanceRef<Image>> = jvm
                    .invoke_virtual(&sis_image, "getObject", "(IZ)Ljavax/microedition/lcdui/Image;", (-1, false))
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_object else {
                    panic!("SISImage.getObject accepted an out-of-range ID");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let target: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (1, 1),
                    )
                    .await?;
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(&target, "getGraphics", "()Ljavax/microedition/lcdui/Graphics;", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &sis_image,
                        "paintFrame",
                        "(Ljavax/microedition/lcdui/Graphics;III)V",
                        (graphics.clone(), 0, 0, 0),
                    )
                    .await?;

                let invalid_frame: JvmResult<()> = jvm
                    .invoke_virtual(
                        &sis_image,
                        "paintFrame",
                        "(Ljavax/microedition/lcdui/Graphics;III)V",
                        (graphics.clone(), 1, 0, 0),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_frame else {
                    panic!("SISImage.paintFrame accepted an out-of-range ID");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let invalid_object: JvmResult<()> = jvm
                    .invoke_virtual(
                        &sis_image,
                        "paintObject",
                        "(Ljavax/microedition/lcdui/Graphics;IIIZ)V",
                        (graphics.clone(), 1, 0, 0, false),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_object else {
                    panic!("SISImage.paintObject accepted an out-of-range ID");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                let _: () = jvm
                    .invoke_virtual(
                        &sis_image,
                        "paintObject",
                        "(Ljavax/microedition/lcdui/Graphics;IIIZ)V",
                        (graphics, 0, 0, 0, false),
                    )
                    .await?;

                let invalid_range: JvmResult<ClassInstanceRef<SISImage>> = jvm
                    .invoke_static("com/skt/m/SISImage", "createSISImage", "([BII)Lcom/skt/m/SISImage;", (data, 2, 2))
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_range else {
                    panic!("SISImage.createSISImage accepted an invalid byte range");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/ArrayIndexOutOfBoundsException"));

                let null_data = ClassInstanceRef::<Array<i8>>::new(None);
                let null_result: JvmResult<i32> = jvm
                    .invoke_static("com/skt/m/SISImage", "getRequiredBufferSize", "([BII)I", (null_data, 0, 0))
                    .await;
                let Err(JavaError::JavaException(exception)) = null_result else {
                    panic!("SISImage.getRequiredBufferSize accepted null data");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                Ok(())
            },
        )
        .unwrap();
    }
}
