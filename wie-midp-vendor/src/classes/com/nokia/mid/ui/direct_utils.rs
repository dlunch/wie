use core::iter;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

use super::DirectGraphics;

// class com.nokia.mid.ui.DirectUtils
#[java_class(name = "com/nokia/mid/ui/DirectUtils", public)]
pub struct DirectUtils;

#[java_methods]
impl DirectUtils {
    #[method(public, static)]
    async fn get_direct_graphics(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<ClassInstanceRef<DirectGraphics>> {
        tracing::debug!("com.nokia.mid.ui.DirectUtils::getDirectGraphics({graphics:?})");

        Ok(jvm
            .new_class(
                "com/nokia/mid/ui/DirectGraphicsImpl",
                "(Ljavax/microedition/lcdui/Graphics;)V",
                (graphics,),
            )
            .await?
            .into())
    }

    #[method(public, static)]
    async fn create_image(jvm: &Jvm, _: &mut WieJvmContext, width: i32, height: i32, argb: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.nokia.mid.ui.DirectUtils::createImage({width}, {height}, {argb})");

        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "image dimensions must be positive")
                .await);
        }
        let count = i64::from(width) * i64::from(height);
        let mut pixels = jvm.instantiate_array("I", count as usize).await?;
        jvm.store_array(&mut pixels, 0, iter::repeat_n(argb, count as usize)).await?;
        let mut image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createRGBImage",
                "([IIIZ)Ljavax/microedition/lcdui/Image;",
                (pixels, width, height, true),
            )
            .await?;
        jvm.put_field(&mut image, "javax/microedition/lcdui/Image", "mutable", "Z", true).await?;
        Ok(image)
    }

    #[method(name = "createImage", public, static)]
    async fn create_image_from_data(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.nokia.mid.ui.DirectUtils::createImage({data:?}, {offset}, {length})");

        let mut image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "([BII)Ljavax/microedition/lcdui/Image;",
                (data, offset, length),
            )
            .await?;
        jvm.put_field(&mut image, "javax/microedition/lcdui/Image", "mutable", "Z", true).await?;
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef};
    use test_utils::run_jvm_test;
    use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

    use crate::classes::com::nokia::mid::ui::direct_graphics::{FLIP_HORIZONTAL, TYPE_INT_888_RGB};

    use super::DirectGraphics;

    #[test]
    fn direct_graphics_draws_into_mutable_image() {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), crate::get_protos().into()]), |jvm| async move {
            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "com/nokia/mid/ui/DirectUtils",
                    "createImage",
                    "(III)Ljavax/microedition/lcdui/Image;",
                    (3, 1, 0xff112233u32 as i32),
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
            let direct: ClassInstanceRef<DirectGraphics> = jvm
                .invoke_static(
                    "com/nokia/mid/ui/DirectUtils",
                    "getDirectGraphics",
                    "(Ljavax/microedition/lcdui/Graphics;)Lcom/nokia/mid/ui/DirectGraphics;",
                    (graphics,),
                )
                .await?;
            let mut pixels: ClassInstanceRef<Array<i32>> = jvm.instantiate_array("I", 3).await?.into();
            jvm.store_array(&mut pixels, 0, [0x445566, 0x778899]).await?;
            let _: () = jvm
                .invoke_virtual(
                    &direct,
                    "com/nokia/mid/ui/DirectGraphics",
                    "drawPixels",
                    "([IZIIIIIIII)V",
                    [
                        pixels.clone().into(),
                        false.into(),
                        0.into(),
                        2.into(),
                        1.into(),
                        0.into(),
                        2.into(),
                        1.into(),
                        FLIP_HORIZONTAL.into(),
                        TYPE_INT_888_RGB.into(),
                    ],
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &direct,
                    "com/nokia/mid/ui/DirectGraphics",
                    "getPixels",
                    "([IIIIIIII)V",
                    (pixels.clone(), 0, 3, 0, 0, 3, 1, TYPE_INT_888_RGB),
                )
                .await?;
            assert_eq!(jvm.load_array::<i32>(&pixels, 0, 3).await?, [0x112233, 0x778899, 0x445566]);
            Ok(())
        })
        .unwrap();
    }
}
