use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_backend::canvas::Color;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::{
    lcdui::{Graphics, Image},
    midlet::MIDlet,
};

// class com.xce.lcdui.XDisplay
pub struct XDisplay;

impl XDisplay {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/XDisplay",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("refresh", "(IIII)V", Self::refresh, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "drawImageEx",
                    "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IILjavax/microedition/lcdui/Image;IIIII)V",
                    Self::draw_image_ex,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "copyLCD",
                    "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IIII)V",
                    Self::copy_lcd,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "IMG_COPY",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "IMG_AND",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "IMG_OR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "IMG_XOR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("width", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("height2", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::<clinit>()");

        jvm.put_static_field("com/xce/lcdui/XDisplay", "IMG_COPY", "I", 0).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "IMG_AND", "I", 1).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "IMG_OR", "I", 2).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "IMG_XOR", "I", 3).await?;

        // TODO: temp
        jvm.put_static_field("com/xce/lcdui/XDisplay", "width", "I", 240).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "height", "I", 320).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "height2", "I", 320).await?;

        Ok(())
    }

    async fn refresh(jvm: &Jvm, context: &mut WieJvmContext, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::refresh({x}, {y}, {width}, {height})");

        let midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;
        let display = MIDlet::display(jvm, &midlet).await?;
        let screen_image: ClassInstanceRef<Image> = jvm
            .get_field(
                &display,
                "javax/microedition/lcdui/Display",
                "screenImage",
                "Ljavax/microedition/lcdui/Image;",
            )
            .await?;
        let image = Image::image(jvm, &screen_image).await?;
        let platform = context.system().platform();
        let screen = platform.screen();
        screen.paint(&*image);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn draw_image_ex(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut graphics: ClassInstanceRef<Graphics>,
        destination: ClassInstanceRef<Image>,
        tx: i32,
        ty: i32,
        source: ClassInstanceRef<Image>,
        sx: i32,
        sy: i32,
        width: i32,
        height: i32,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "com.xce.lcdui.XDisplay::drawImageEx({graphics:?}, {destination:?}, {tx}, {ty}, {source:?}, {sx}, {sy}, {width}, {height}, {mode})"
        );

        if graphics.is_null() || source.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics or source is null").await);
        }
        let copy: i32 = jvm.get_static_field("com/xce/lcdui/XDisplay", "IMG_COPY", "I").await?;
        let and: i32 = jvm.get_static_field("com/xce/lcdui/XDisplay", "IMG_AND", "I").await?;
        let or: i32 = jvm.get_static_field("com/xce/lcdui/XDisplay", "IMG_OR", "I").await?;
        let xor: i32 = jvm.get_static_field("com/xce/lcdui/XDisplay", "IMG_XOR", "I").await?;
        if ![copy, and, or, xor].contains(&mode) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid image mode").await);
        }
        if width <= 0 || height <= 0 {
            return Ok(());
        }

        let translate_x: i32 = jvm.get_field(&graphics, "javax/microedition/lcdui/Graphics", "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "javax/microedition/lcdui/Graphics", "translateY", "I").await?;
        let tx = tx.wrapping_add(translate_x);
        let ty = ty.wrapping_add(translate_y);
        let clip = Graphics::clip(jvm, &graphics).await?;
        let destination = if destination.is_null() {
            Graphics::image(jvm, &mut graphics).await?
        } else {
            destination
        };
        let source = Image::image(jvm, &source).await?;
        let mut canvas = Image::canvas(jvm, &destination).await?;
        if mode == copy || mode == xor {
            canvas.set_xor_mode(mode == xor);
            canvas.draw(tx, ty, width as _, height as _, &*source, sx, sy, clip);
            return Ok(());
        }

        let x_start = 0i64.max(-i64::from(tx)).max(-i64::from(sx)).max(i64::from(clip.x) - i64::from(tx));
        let y_start = 0i64.max(-i64::from(ty)).max(-i64::from(sy)).max(i64::from(clip.y) - i64::from(ty));
        let x_end = i64::from(width)
            .min(i64::from(canvas.image().width()) - i64::from(tx))
            .min(i64::from(source.width()) - i64::from(sx))
            .min(i64::from(clip.x) + i64::from(clip.width) - i64::from(tx));
        let y_end = i64::from(height)
            .min(i64::from(canvas.image().height()) - i64::from(ty))
            .min(i64::from(source.height()) - i64::from(sy))
            .min(i64::from(clip.y) + i64::from(clip.height) - i64::from(ty));

        // Traverse away from overlapping source pixels, as the backend draw path does.
        for row in 0..y_end - y_start {
            let y = if ty > sy { y_end - 1 - row } else { y_start + row };
            for column in 0..x_end - x_start {
                let x = if tx > sx { x_end - 1 - column } else { x_start + column };
                let source = source.get_pixel((i64::from(sx) + x) as i32, (i64::from(sy) + y) as i32);
                if source.a == 0 {
                    continue;
                }
                let dx = (i64::from(tx) + x) as i32;
                let dy = (i64::from(ty) + y) as i32;
                let destination = canvas.image().get_pixel(dx, dy);
                let color = if mode == and {
                    Color {
                        a: 255,
                        r: destination.r & source.r,
                        g: destination.g & source.g,
                        b: destination.b & source.b,
                    }
                } else {
                    Color {
                        a: 255,
                        r: destination.r | source.r,
                        g: destination.g | source.g,
                        b: destination.b | source.b,
                    }
                };
                canvas.put_pixel(dx, dy, color, clip);
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn copy_lcd(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XDisplay::copyLCD({graphics:?}, {image:?}, {x}, {y}, {width}, {height})",);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicU32, Ordering};

    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};

    use super::*;

    #[test]
    fn image_regions_apply_modes_translation_clipping_and_overlap() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [XDisplay::as_proto()].into()]),
            |jvm| async move {
                let mut pixels = jvm.instantiate_array("I", 8).await?;
                jvm.store_array(
                    &mut pixels,
                    0,
                    [
                        0xffabcdefu32 as i32,
                        0,
                        0,
                        0,
                        0xffee1122u32 as i32,
                        0xff0ff055u32 as i32,
                        0x00ffffff,
                        0xffee1122u32 as i32,
                    ],
                )
                .await?;
                let source: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createRGBImage",
                        "([IIIZ)Ljavax/microedition/lcdui/Image;",
                        (pixels, 4, 2, true),
                    )
                    .await?;
                for (mode, expected) in [(0, 0x0ff055), (1, 0x023054), (2, 0x1ff457), (3, 0x1dc403)] {
                    for explicit_destination in [false, true] {
                        let destination: ClassInstanceRef<Image> = jvm
                            .invoke_static(
                                "javax/microedition/lcdui/Image",
                                "createImage",
                                "(II)Ljavax/microedition/lcdui/Image;",
                                (5, 3),
                            )
                            .await?;
                        let target_graphics: ClassInstanceRef<Graphics> = jvm
                            .invoke_virtual(
                                &destination,
                                "javax/microedition/lcdui/Image",
                                "getGraphics",
                                "()Ljavax/microedition/lcdui/Graphics;",
                                (),
                            )
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&target_graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x123456,))
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&target_graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 5, 3))
                            .await?;
                        let graphics_image: ClassInstanceRef<Image> = if explicit_destination {
                            jvm.invoke_static(
                                "javax/microedition/lcdui/Image",
                                "createImage",
                                "(II)Ljavax/microedition/lcdui/Image;",
                                (5, 3),
                            )
                            .await?
                        } else {
                            destination.clone()
                        };
                        let graphics: ClassInstanceRef<Graphics> = jvm
                            .invoke_virtual(
                                &graphics_image,
                                "javax/microedition/lcdui/Image",
                                "getGraphics",
                                "()Ljavax/microedition/lcdui/Graphics;",
                                (),
                            )
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (1, 1))
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (1, 0, 2, 1))
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setXORMode", "(Z)V", (true,))
                            .await?;
                        let argument_destination = if explicit_destination {
                            destination.clone()
                        } else {
                            ClassInstanceRef::new(None)
                        };
                        let _: () = jvm
                            .invoke_static(
                                "com/xce/lcdui/XDisplay",
                                "drawImageEx",
                                "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IILjavax/microedition/lcdui/Image;IIIII)V",
                                vec![
                                    graphics.clone().into(),
                                    argument_destination.into(),
                                    0.into(),
                                    0.into(),
                                    source.clone().into(),
                                    0.into(),
                                    1.into(),
                                    4.into(),
                                    1.into(),
                                    mode.into(),
                                ],
                            )
                            .await?;
                        let image = Image::image(&jvm, &destination).await?;
                        for y in 0..3 {
                            for x in 0..5 {
                                let color = image.get_pixel(x, y);
                                let rgb = (i32::from(color.r) << 16) | (i32::from(color.g) << 8) | i32::from(color.b);
                                assert_eq!(
                                    rgb,
                                    if (x, y) == (2, 1) { expected } else { 0x123456 },
                                    "mode={mode}, explicit={explicit_destination}, pixel=({x},{y})"
                                );
                            }
                        }
                        if explicit_destination {
                            let pixel = Image::image(&jvm, &graphics_image).await?.get_pixel(2, 1);
                            assert_eq!((pixel.r, pixel.g, pixel.b), (255, 255, 255));
                        }
                        assert_eq!(
                            jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getTranslateX", "()I", ())
                                .await?,
                            1
                        );
                        assert_eq!(
                            jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                                .await?,
                            1
                        );
                        assert!(
                            jvm.get_field::<bool>(&graphics, "javax/microedition/lcdui/Graphics", "xorMode", "Z")
                                .await?
                        );
                    }
                }
                for (mode, expected) in [
                    (0, [0x0f, 0x0f, 0x33, 0x55]),
                    (1, [0x0f, 0x03, 0x11, 0x00]),
                    (2, [0x0f, 0x3f, 0x77, 0xff]),
                    (3, [0x0f, 0x3c, 0x66, 0xff]),
                ] {
                    let image: ClassInstanceRef<Image> = jvm
                        .invoke_static(
                            "javax/microedition/lcdui/Image",
                            "createImage",
                            "(II)Ljavax/microedition/lcdui/Image;",
                            (4, 1),
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
                    let mut pixels = jvm.instantiate_array("I", 4).await?;
                    jvm.store_array(&mut pixels, 0, [0x0f0f0f, 0x333333, 0x555555, 0xaaaaaa]).await?;
                    let _: () = jvm
                        .invoke_virtual(
                            &graphics,
                            "javax/microedition/lcdui/Graphics",
                            "drawRGB",
                            "([IIIIIIIZ)V",
                            (pixels, 0, 4, 0, 0, 4, 1, false),
                        )
                        .await?;
                    let _: () = jvm
                        .invoke_static(
                            "com/xce/lcdui/XDisplay",
                            "drawImageEx",
                            "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IILjavax/microedition/lcdui/Image;IIIII)V",
                            vec![
                                graphics.into(),
                                image.clone().into(),
                                1.into(),
                                0.into(),
                                image.clone().into(),
                                0.into(),
                                0.into(),
                                4.into(),
                                1.into(),
                                mode.into(),
                            ],
                        )
                        .await?;
                    let image = Image::image(&jvm, &image).await?;
                    for (x, expected) in expected.into_iter().enumerate() {
                        let pixel = image.get_pixel(x as i32, 0);
                        assert_eq!((pixel.r, pixel.g, pixel.b), (expected, expected, expected), "overlap mode={mode}, x={x}");
                    }
                }
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn refresh_presents_shared_pixels_without_consuming_pending_repaint() {
        let midlet_proto = WieJavaClassProto {
            name: "test/RefreshMidlet",
            parent_class: Some("javax/microedition/midlet/MIDlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        let paints = Arc::new(AtomicU32::new(0));
        let observed_paints = paints.clone();
        let platform = TestPlatform::with_paint_handler(move |image| {
            let pixel = image.get_pixel(3, 4);
            assert_eq!((pixel.r, pixel.g, pixel.b), (0x12, 0x34, 0x56));
            observed_paints.fetch_add(1, Ordering::SeqCst);
        });
        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), [XDisplay::as_proto(), midlet_proto].into()]),
            Box::new(platform),
            move |jvm, _system| async move {
                let midlet: ClassInstanceRef<MIDlet> = jvm.instantiate_class("test/RefreshMidlet").await?.into();
                let _: () = jvm
                    .invoke_special(&midlet, "javax/microedition/midlet/MIDlet", "<init>", "()V", ())
                    .await?;
                let display = MIDlet::display(&jvm, &midlet).await?;
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "getScreenGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x123456,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (3, 4, 2, 1))
                    .await?;
                for pending in [false, true] {
                    if pending {
                        let _: () = jvm
                            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, 320, 240))
                            .await?;
                    }
                    let _: () = jvm.invoke_static("com/xce/lcdui/XDisplay", "refresh", "(IIII)V", (3, 4, 2, 1)).await?;
                    assert_eq!(paints.load(Ordering::SeqCst), if pending { 2 } else { 1 });
                    assert_eq!(
                        jvm.get_field::<bool>(&display, "javax/microedition/lcdui/Display", "repaintPending", "Z")
                            .await?,
                        pending
                    );
                }
                Ok(())
            },
        )
        .unwrap();
    }
}
