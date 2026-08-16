use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics as MidpGraphics, Image as MidpImage};

use crate::classes::org::kwis::msp::lcdui::{Graphics, ImageObserver};

// class org.kwis.msp.lcdui.Image
pub struct Image;

impl Image {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Image",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init_empty, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Image;)V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new(
                    "loadImage",
                    "(Ljava/lang/String;Lorg/kwis/msp/lcdui/ImageObserver;)Lorg/kwis/msp/lcdui/Image;",
                    Self::load_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(II)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_name,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "([BII)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_data,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Lorg/kwis/msp/lcdui/Image;)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getGraphics",
                    "()Lorg/kwis/msp/lcdui/Graphics;",
                    Self::get_graphics,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isMutable", "()Z", Self::is_mutable, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isAnimated", "()Z", Self::is_animated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("play", "(Lorg/kwis/msp/lcdui/ImageObserver;)V", Self::play, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "stopImage",
                    "(Lorg/kwis/msp/lcdui/ImageObserver;)V",
                    Self::stop_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "drawImage",
                    "(Lorg/kwis/msp/lcdui/Image;IIIIIIII)V",
                    Self::draw_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "createSubImage",
                    "(IIIIZ)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_sub_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setTransparentColor", "(I)V", Self::set_transparent_color, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("midpImage", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("mutable", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init_empty(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Image::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "midpImage", "Ljavax/microedition/lcdui/Image;", None).await?;
        jvm.put_field(&mut this, "mutable", "Z", false).await
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Image>, image: ClassInstanceRef<MidpImage>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Image::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "midpImage", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(&mut this, "mutable", "Z", false).await?;

        Ok(())
    }

    async fn load_image(
        _: &Jvm,
        _: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        observer: ClassInstanceRef<ImageObserver>,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Image::loadImage({name:?}, {observer:?})");

        Ok(None.into())
    }

    async fn create_image(jvm: &Jvm, _: &mut WieJvmContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({width}, {height})");

        let midp_image: ClassInstanceRef<MidpImage> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        let mut instance: ClassInstanceRef<Image> = jvm
            .new_class("org/kwis/msp/lcdui/Image", "(Ljavax/microedition/lcdui/Image;)V", (midp_image,))
            .await?
            .into();
        jvm.put_field(&mut instance, "mutable", "Z", true).await?;

        Ok(instance)
    }

    async fn create_image_from_name(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({name:?})");

        let midp_image: ClassInstanceRef<MidpImage> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
                (name,),
            )
            .await?;

        let instance = jvm
            .new_class("org/kwis/msp/lcdui/Image", "(Ljavax/microedition/lcdui/Image;)V", (midp_image,))
            .await?;

        Ok(instance.into())
    }

    async fn create_image_from_data(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        data: ClassInstanceRef<Array<i8>>,
        image_offset: i32,
        image_length: i32,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({data:?}, {image_offset}, {image_length})");

        let midp_image: ClassInstanceRef<MidpImage> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "([BII)Ljavax/microedition/lcdui/Image;",
                (data, image_offset, image_length),
            )
            .await?;

        let instance = jvm
            .new_class("org/kwis/msp/lcdui/Image", "(Ljavax/microedition/lcdui/Image;)V", (midp_image,))
            .await?;

        Ok(instance.into())
    }

    async fn create_image_from_image(jvm: &Jvm, _: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({image:?})");

        let midp_image: ClassInstanceRef<MidpImage> = jvm.get_field(&image, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;
        let midp_image_clone: ClassInstanceRef<MidpImage> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(Ljavax/microedition/lcdui/Image;)Ljavax/microedition/lcdui/Image;",
                (midp_image,),
            )
            .await?;

        let instance = jvm
            .new_class("org/kwis/msp/lcdui/Image", "(Ljavax/microedition/lcdui/Image;)V", (midp_image_clone,))
            .await?;

        Ok(instance.into())
    }

    async fn get_graphics(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>) -> JvmResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getGraphics({this:?})");

        let midp_image: ClassInstanceRef<MidpImage> = jvm.get_field(&this, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;

        let midp_graphics: ClassInstanceRef<MidpGraphics> = jvm
            .invoke_virtual(
                &midp_image,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;

        let instance = jvm
            .new_class("org/kwis/msp/lcdui/Graphics", "(Ljavax/microedition/lcdui/Graphics;)V", (midp_graphics,))
            .await?;

        Ok(instance.into())
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getWidth({this:?})");

        let midp_image: ClassInstanceRef<MidpImage> = jvm.get_field(&this, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;

        jvm.invoke_virtual(&midp_image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getHeight({this:?})");

        let midp_image: ClassInstanceRef<MidpImage> = jvm.get_field(&this, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;

        jvm.invoke_virtual(&midp_image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
            .await
    }

    async fn is_mutable(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.Image::isMutable({this:?})");

        jvm.get_field(&this, "mutable", "Z").await
    }

    async fn is_animated(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.Image::isAnimated({this:?})");

        Ok(false)
    }

    async fn play(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>, observer: ClassInstanceRef<ImageObserver>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Image::play({this:?}, {observer:?})");

        Ok(())
    }

    async fn stop(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Image::stop({this:?})");

        Ok(())
    }

    async fn stop_image(_: &Jvm, _: &mut WieJvmContext, observer: ClassInstanceRef<ImageObserver>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Image::stopImage({observer:?})");

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn draw_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Image>,
        image: ClassInstanceRef<Image>,
        src_x: i32,
        src_y: i32,
        src_width: i32,
        src_height: i32,
        dest_x: i32,
        dest_y: i32,
        transform: i32,
        anchor: i32,
    ) -> JvmResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Image::drawImage({this:?}, {image:?}, {src_x}, {src_y}, {src_width}, {src_height}, {dest_x}, {dest_y}, {transform}, {anchor})"
        );

        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }

        let target: ClassInstanceRef<MidpImage> = jvm.get_field(&this, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;
        let source: ClassInstanceRef<MidpImage> = jvm.get_field(&image, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;
        let graphics: ClassInstanceRef<MidpGraphics> = jvm
            .invoke_virtual(
                &target,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;
        jvm.invoke_virtual(
            &graphics,
            "javax/microedition/lcdui/Graphics",
            "drawRegion",
            "(Ljavax/microedition/lcdui/Image;IIIIIIII)V",
            [
                source.into(),
                src_x.into(),
                src_y.into(),
                src_width.into(),
                src_height.into(),
                transform.into(),
                dest_x.into(),
                dest_y.into(),
                anchor.into(),
            ],
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_sub_image(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        mutable: bool,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Image::createSubImage({this:?}, {x}, {y}, {width}, {height}, {mutable})");

        Ok(None.into())
    }

    async fn set_transparent_color(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>, rgb: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.Image::setTransparentColor({this:?}, {rgb})");

        Ok(())
    }

    pub async fn midp_image(jvm: &Jvm, this: &ClassInstanceRef<Image>) -> JvmResult<ClassInstanceRef<MidpImage>> {
        jvm.get_field(this, "midpImage", "Ljavax/microedition/lcdui/Image;").await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::ClassInstanceRef;
    use test_utils::run_jvm_test;
    use wie_midp::classes::javax::microedition::lcdui::Image as MidpImage;
    use wie_util::Result;

    use crate::{
        classes::org::kwis::msp::lcdui::{Graphics, Image},
        get_protos,
    };

    #[test]
    fn test_mutability_and_region_draw() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let source: ClassInstanceRef<Image> = jvm
                .invoke_static("org/kwis/msp/lcdui/Image", "createImage", "(II)Lorg/kwis/msp/lcdui/Image;", (1, 1))
                .await?;
            let target: ClassInstanceRef<Image> = jvm
                .invoke_static("org/kwis/msp/lcdui/Image", "createImage", "(II)Lorg/kwis/msp/lcdui/Image;", (2, 1))
                .await?;
            assert!(
                jvm.invoke_virtual::<_, bool>(&source, "org/kwis/msp/lcdui/Image", "isMutable", "()Z", ())
                    .await?
            );

            let clone: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "org/kwis/msp/lcdui/Image",
                    "createImage",
                    "(Lorg/kwis/msp/lcdui/Image;)Lorg/kwis/msp/lcdui/Image;",
                    (source.clone(),),
                )
                .await?;
            assert!(
                !jvm.invoke_virtual::<_, bool>(&clone, "org/kwis/msp/lcdui/Image", "isMutable", "()Z", ())
                    .await?
            );
            assert!(
                !jvm.invoke_virtual::<_, bool>(&source, "org/kwis/msp/lcdui/Image", "isAnimated", "()Z", ())
                    .await?
            );

            let graphics: ClassInstanceRef<Graphics> = jvm
                .invoke_virtual(&source, "org/kwis/msp/lcdui/Image", "getGraphics", "()Lorg/kwis/msp/lcdui/Graphics;", ())
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "setColor", "(I)V", (0xff0000,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 1, 1))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &target,
                    "org/kwis/msp/lcdui/Image",
                    "drawImage",
                    "(Lorg/kwis/msp/lcdui/Image;IIIIIIII)V",
                    [
                        source.into(),
                        0.into(),
                        0.into(),
                        1.into(),
                        1.into(),
                        1.into(),
                        0.into(),
                        0.into(),
                        20.into(),
                    ],
                )
                .await?;

            let target_midp = Image::midp_image(&jvm, &target).await?;
            let target_backend = MidpImage::image(&jvm, &target_midp).await?;
            let pixel = target_backend.get_pixel(1, 0);
            assert_eq!((pixel.r, pixel.g, pixel.b), (0xff, 0, 0));

            Ok(())
        })
    }
}
