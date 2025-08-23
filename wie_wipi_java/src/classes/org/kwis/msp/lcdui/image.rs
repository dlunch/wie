use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics as MidpGraphics, Image as MidpImage};

use crate::classes::org::kwis::msp::lcdui::Graphics;

// class org.kwis.msp.lcdui.Image
pub struct Image;

impl Image {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Image",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Image;)V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "createImage",
                    "(II)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_name,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "([BII)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_data,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Lorg/kwis/msp/lcdui/Image;)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_image,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getGraphics", "()Lorg/kwis/msp/lcdui/Graphics;", Self::get_graphics, Default::default()),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("midpImage", "Ljavax/microedition/lcdui/Image;", Default::default())],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Image>, image: ClassInstanceRef<MidpImage>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Image::<init>({:?})", &this);

        jvm.put_field(&mut this, "midpImage", "Ljavax/microedition/lcdui/Image;", image).await?;

        Ok(())
    }

    async fn create_image(jvm: &Jvm, _: &mut WieJvmContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({}, {})", width, height);

        let midp_image: ClassInstanceRef<MidpImage> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        let instance = jvm
            .new_class("org/kwis/msp/lcdui/Image", "(Ljavax/microedition/lcdui/Image;)V", (midp_image,))
            .await?;

        Ok(instance.into())
    }

    async fn create_image_from_name(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?})", &name);

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
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?}, {}, {})", &data, image_offset, image_length);

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
        tracing::debug!("org.kwis.msp.lcdui.Image::getGraphics({:?})", &this);

        let midp_image: ClassInstanceRef<MidpImage> = jvm.get_field(&this, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;

        let midp_graphics: ClassInstanceRef<MidpGraphics> = jvm
            .invoke_virtual(&midp_image, "getGraphics", "()Ljavax/microedition/lcdui/Graphics;", ())
            .await?;

        let instance = jvm
            .new_class("org/kwis/msp/lcdui/Graphics", "(Ljavax/microedition/lcdui/Graphics;)V", (midp_graphics,))
            .await?;

        Ok(instance.into())
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getWidth({:?})", &this);

        let midp_image: ClassInstanceRef<MidpImage> = jvm.get_field(&this, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;

        jvm.invoke_virtual(&midp_image, "getWidth", "()I", ()).await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Image>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getHeight({:?})", &this);

        let midp_image: ClassInstanceRef<MidpImage> = jvm.get_field(&this, "midpImage", "Ljavax/microedition/lcdui/Image;").await?;

        jvm.invoke_virtual(&midp_image, "getHeight", "()I", ()).await
    }

    pub async fn midp_image(jvm: &Jvm, this: &ClassInstanceRef<Image>) -> JvmResult<ClassInstanceRef<MidpImage>> {
        jvm.get_field(this, "midpImage", "Ljavax/microedition/lcdui/Image;").await
    }
}
