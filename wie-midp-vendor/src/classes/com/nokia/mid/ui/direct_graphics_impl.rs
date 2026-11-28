use alloc::vec::Vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

use super::direct_graphics::{FLIP_HORIZONTAL, FLIP_VERTICAL, ROTATE_90, ROTATE_180, ROTATE_270, TYPE_INT_888_RGB, TYPE_INT_8888_ARGB};

// class com.nokia.mid.ui.DirectGraphicsImpl
#[java_class(name = "com/nokia/mid/ui/DirectGraphicsImpl", implements("com/nokia/mid/ui/DirectGraphics"), final)]
pub struct DirectGraphicsImpl {
    #[field(private)]
    graphics: ClassInstanceRef<Graphics>,
}

#[allow(clippy::too_many_arguments)]
#[java_methods]
impl DirectGraphicsImpl {
    #[method(constructor)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.DirectGraphicsImpl::<init>({this:?}, {graphics:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(
            &mut this,
            "com/nokia/mid/ui/DirectGraphicsImpl",
            "graphics",
            "Ljavax/microedition/lcdui/Graphics;",
            graphics,
        )
        .await
    }

    #[method(name = "setARGBColor", public)]
    async fn set_argb_color(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, argb: i32) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.DirectGraphicsImpl::setARGBColor({this:?}, {argb})");

        let graphics: ClassInstanceRef<Graphics> = jvm
            .get_field(
                &this,
                "com/nokia/mid/ui/DirectGraphicsImpl",
                "graphics",
                "Ljavax/microedition/lcdui/Graphics;",
            )
            .await?;
        // MIDP drawing uses only the RGB part of the Nokia color.
        jvm.invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (argb & 0xffffff,))
            .await
    }

    #[method(public)]
    async fn get_alpha_component(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.nokia.mid.ui.DirectGraphicsImpl::getAlphaComponent({this:?})");

        Ok(0xff)
    }

    #[method(public)]
    async fn get_native_pixel_format(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.nokia.mid.ui.DirectGraphicsImpl::getNativePixelFormat({this:?})");

        Ok(TYPE_INT_8888_ARGB)
    }

    #[method(public)]
    async fn draw_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        anchor: i32,
        manipulation: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.DirectGraphicsImpl::drawImage({this:?}, {image:?}, {x}, {y}, {anchor}, {manipulation})");

        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        // MIDP Sprite transforms after a counterclockwise rotation and each possible pair of Nokia flip flags.
        let transforms = match manipulation & !(FLIP_HORIZONTAL | FLIP_VERTICAL) {
            0 => [0, 2, 1, 3],
            ROTATE_90 => [6, 7, 4, 5],
            ROTATE_180 => [3, 1, 2, 0],
            ROTATE_270 => [5, 4, 7, 6],
            _ => return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid manipulation").await),
        };
        let horizontal = usize::from(manipulation & FLIP_HORIZONTAL != 0);
        let vertical = usize::from(manipulation & FLIP_VERTICAL != 0);
        let transform = transforms[horizontal + 2 * vertical];
        let graphics: ClassInstanceRef<Graphics> = jvm
            .get_field(
                &this,
                "com/nokia/mid/ui/DirectGraphicsImpl",
                "graphics",
                "Ljavax/microedition/lcdui/Graphics;",
            )
            .await?;
        let width: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
            .await?;
        jvm.invoke_virtual(
            &graphics,
            "javax/microedition/lcdui/Graphics",
            "drawRegion",
            "(Ljavax/microedition/lcdui/Image;IIIIIIII)V",
            [
                image.into(),
                0.into(),
                0.into(),
                width.into(),
                height.into(),
                transform.into(),
                x.into(),
                y.into(),
                anchor.into(),
            ],
        )
        .await
    }

    #[method(public)]
    async fn draw_triangle(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        argb: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.DirectGraphicsImpl::drawTriangle({this:?}, {x1}, {y1}, {x2}, {y2}, {x3}, {y3}, {argb})");

        Self::draw_outline(jvm, &this, &[x1, x2, x3], &[y1, y2, y3], argb).await
    }

    #[method(public)]
    async fn draw_polygon(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        xs: ClassInstanceRef<Array<i32>>,
        x_offset: i32,
        ys: ClassInstanceRef<Array<i32>>,
        y_offset: i32,
        count: i32,
        argb: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.DirectGraphicsImpl::drawPolygon({this:?}, {xs:?}, {x_offset}, {ys:?}, {y_offset}, {count}, {argb})");

        if xs.is_null() || ys.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "coordinate array is null").await);
        }
        let xs = jvm.load_array(&xs, x_offset as _, count as _).await?;
        let ys = jvm.load_array(&ys, y_offset as _, count as _).await?;
        Self::draw_outline(jvm, &this, &xs, &ys, argb).await
    }

    async fn draw_outline(jvm: &Jvm, this: &ClassInstanceRef<Self>, xs: &[i32], ys: &[i32], argb: i32) -> JvmResult<()> {
        if xs.is_empty() {
            return Ok(());
        }
        let graphics: ClassInstanceRef<Graphics> = jvm
            .get_field(
                this,
                "com/nokia/mid/ui/DirectGraphicsImpl",
                "graphics",
                "Ljavax/microedition/lcdui/Graphics;",
            )
            .await?;
        let previous_color: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getColor", "()I", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (argb & 0xffffff,))
            .await?;
        let mut previous = xs.len() - 1;
        for i in 0..xs.len() {
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawLine",
                    "(IIII)V",
                    (xs[previous], ys[previous], xs[i], ys[i]),
                )
                .await?;
            previous = i;
        }
        jvm.invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (previous_color,))
            .await
    }

    #[method(public)]
    async fn fill_triangle(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        argb: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.nokia.mid.ui.DirectGraphicsImpl::fillTriangle({this:?}, {x1}, {y1}, {x2}, {y2}, {x3}, {y3}, {argb})");

        let graphics: ClassInstanceRef<Graphics> = jvm
            .get_field(
                &this,
                "com/nokia/mid/ui/DirectGraphicsImpl",
                "graphics",
                "Ljavax/microedition/lcdui/Graphics;",
            )
            .await?;
        let previous_color: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getColor", "()I", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (argb & 0xffffff,))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "fillTriangle",
                "(IIIIII)V",
                (x1, y1, x2, y2, x3, y3),
            )
            .await?;
        jvm.invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (previous_color,))
            .await
    }

    #[method(public)]
    async fn fill_polygon(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        xs: ClassInstanceRef<Array<i32>>,
        x_offset: i32,
        ys: ClassInstanceRef<Array<i32>>,
        y_offset: i32,
        count: i32,
        argb: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.nokia.mid.ui.DirectGraphicsImpl::fillPolygon({this:?}, {xs:?}, {x_offset}, {ys:?}, {y_offset}, {count}, {argb})");

        Ok(())
    }

    #[method(name = "drawPixels", public)]
    async fn draw_int_pixels(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i32>>,
        transparency: bool,
        offset: i32,
        stride: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "com.nokia.mid.ui.DirectGraphicsImpl::drawPixels({this:?}, {pixels:?}, {transparency}, {offset}, {stride}, {x}, {y}, {width}, {height}, {manipulation}, {format})"
        );

        if !matches!(format, TYPE_INT_888_RGB | TYPE_INT_8888_ARGB) || width < 0 || height < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid pixel format or size").await);
        }
        let graphics: ClassInstanceRef<Graphics> = jvm
            .get_field(
                &this,
                "com/nokia/mid/ui/DirectGraphicsImpl",
                "graphics",
                "Ljavax/microedition/lcdui/Graphics;",
            )
            .await?;
        let alpha = transparency && format == TYPE_INT_8888_ARGB;
        if manipulation == 0 || width == 0 || height == 0 {
            return jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawRGB",
                    "([IIIIIIIZ)V",
                    (pixels, offset, stride, x, y, width, height, alpha),
                )
                .await;
        }
        if pixels.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "pixels is null").await);
        }
        let mut packed = Vec::new();
        for row in 0..height {
            let start = i64::from(offset) + i64::from(row) * i64::from(stride);
            let start = usize::try_from(start).unwrap_or(usize::MAX);
            packed.extend(jvm.load_array::<i32>(&pixels, start, width as usize).await?);
        }
        let mut rgb = jvm.instantiate_array("I", packed.len()).await?;
        jvm.store_array(&mut rgb, 0, packed).await?;
        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createRGBImage",
                "([IIIZ)Ljavax/microedition/lcdui/Image;",
                (rgb, width, height, alpha),
            )
            .await?;
        jvm.invoke_virtual(
            &this,
            "com/nokia/mid/ui/DirectGraphicsImpl",
            "drawImage",
            "(Ljavax/microedition/lcdui/Image;IIII)V",
            (image, x, y, 0, manipulation),
        )
        .await
    }

    #[method(name = "getPixels", public)]
    async fn get_int_pixels(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mut pixels: ClassInstanceRef<Array<i32>>,
        offset: i32,
        stride: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "com.nokia.mid.ui.DirectGraphicsImpl::getPixels({this:?}, {pixels:?}, {offset}, {stride}, {x}, {y}, {width}, {height}, {format})"
        );

        if !matches!(format, TYPE_INT_888_RGB | TYPE_INT_8888_ARGB) || width < 0 || height < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid pixel format or size").await);
        }
        let mut graphics: ClassInstanceRef<Graphics> = jvm
            .get_field(
                &this,
                "com/nokia/mid/ui/DirectGraphicsImpl",
                "graphics",
                "Ljavax/microedition/lcdui/Graphics;",
            )
            .await?;
        let image = Graphics::image(jvm, &mut graphics).await?;
        let translate_x: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getTranslateX", "()I", ())
            .await?;
        let translate_y: i32 = jvm
            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getTranslateY", "()I", ())
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &image,
                "javax/microedition/lcdui/Image",
                "getRGB",
                "([IIIIIII)V",
                (pixels.clone(), offset, stride, x + translate_x, y + translate_y, width, height),
            )
            .await?;
        if format == TYPE_INT_888_RGB && width > 0 {
            for row in 0..height {
                let start = (i64::from(offset) + i64::from(row) * i64::from(stride)) as usize;
                let mut values = jvm.load_array::<i32>(&pixels, start, width as _).await?;
                for value in &mut values {
                    *value &= 0x00ff_ffff;
                }
                jvm.store_array(&mut pixels, start, values).await?;
            }
        }
        Ok(())
    }

    #[method(name = "drawPixels", public)]
    async fn draw_byte_pixels(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i8>>,
        mask: ClassInstanceRef<Array<i8>>,
        offset: i32,
        stride: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub com.nokia.mid.ui.DirectGraphicsImpl::drawPixels({this:?}, {pixels:?}, {mask:?}, {offset}, {stride}, {x}, {y}, {width}, {height}, {manipulation}, {format})"
        );

        Ok(())
    }

    #[method(name = "getPixels", public)]
    async fn get_byte_pixels(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i8>>,
        mask: ClassInstanceRef<Array<i8>>,
        offset: i32,
        stride: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub com.nokia.mid.ui.DirectGraphicsImpl::getPixels({this:?}, {pixels:?}, {mask:?}, {offset}, {stride}, {x}, {y}, {width}, {height}, {format})"
        );

        Ok(())
    }

    #[method(name = "drawPixels", public)]
    async fn draw_short_pixels(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i16>>,
        transparency: bool,
        offset: i32,
        stride: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        manipulation: i32,
        format: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub com.nokia.mid.ui.DirectGraphicsImpl::drawPixels({this:?}, {pixels:?}, {transparency}, {offset}, {stride}, {x}, {y}, {width}, {height}, {manipulation}, {format})"
        );

        Ok(())
    }

    #[method(name = "getPixels", public)]
    async fn get_short_pixels(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        pixels: ClassInstanceRef<Array<i16>>,
        offset: i32,
        stride: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: i32,
    ) -> JvmResult<()> {
        tracing::warn!(
            "stub com.nokia.mid.ui.DirectGraphicsImpl::getPixels({this:?}, {pixels:?}, {offset}, {stride}, {x}, {y}, {width}, {height}, {format})"
        );

        Ok(())
    }
}
