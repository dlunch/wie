use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    cell::Ref,
    ops::{Deref, DerefMut},
};

use jvm::{ClassInstanceRef, JavaValue};

use wie_backend::canvas::{create_canvas, decode_image, Canvas, Image as BackendImage, PixelFormat};

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::{JvmArrayClassInstanceProxy, JvmClassInstanceProxy},
    r#impl::{java::lang::String, org::kwis::msp::lcdui::Graphics},
    JavaFieldAccessFlag,
};

// class org.kwis.msp.lcdui.Image
pub struct Image {}

impl Image {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "createImage",
                    "(II)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image,
                    JavaMethodFlag::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_file,
                    JavaMethodFlag::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "([BII)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_bytes,
                    JavaMethodFlag::STATIC,
                ),
                JavaMethodProto::new("getGraphics", "()Lorg/kwis/msp/lcdui/Graphics;", Self::get_graphics, JavaMethodFlag::NONE),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, JavaMethodFlag::NONE),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, JavaMethodFlag::NONE),
            ],
            fields: vec![
                JavaFieldProto::new("w", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("h", "I", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("imgData", "[B", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("bpl", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Image>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Image::<init>({:?})", &this);

        Ok(())
    }

    async fn create_image(context: &mut dyn JavaContext, width: i32, height: i32) -> JavaResult<JvmClassInstanceProxy<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({}, {})", width, height);

        let instance = context.jvm().instantiate_class("org/kwis/msp/lcdui/Image").await?;
        context
            .jvm()
            .invoke_method(&instance, "org/kwis/msp/lcdui/Image", "<init>", "()V", &[])
            .await?;

        let bytes_per_pixel = 4;

        Self::create_image_instance(
            context,
            width as _,
            height as _,
            &vec![0; (width * height * bytes_per_pixel) as usize],
            bytes_per_pixel as _,
        )
        .await
    }

    #[allow(clippy::await_holding_refcell_ref)] // We manually drop Ref
    async fn create_image_from_file(context: &mut dyn JavaContext, name: JvmClassInstanceProxy<String>) -> JavaResult<JvmClassInstanceProxy<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?})", &name);

        let name = String::to_rust_string(context, &name.class_instance.unwrap())?;
        let normalized_name = if let Some(x) = name.strip_prefix('/') { x } else { &name };

        let id = context.backend().resource().id(normalized_name).unwrap();
        let backend1 = context.backend().clone();
        let image_data = Ref::map(backend1.resource(), |x| x.data(id));

        let image = decode_image(&image_data)?;
        drop(image_data);

        Self::create_image_instance(context, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn create_image_from_bytes(
        context: &mut dyn JavaContext,
        data: JvmArrayClassInstanceProxy<i8>,
        offset: i32,
        length: i32,
    ) -> JavaResult<JvmClassInstanceProxy<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?}, {}, {})", &data, offset, length);

        let image_data = context.jvm().load_array(&data.class_instance.unwrap(), offset as _, length as _)?;
        let image_data = image_data.into_iter().map(|x| x.as_byte() as u8).collect::<Vec<_>>();
        let image = decode_image(&image_data)?;

        Self::create_image_instance(context, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn get_graphics(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JvmClassInstanceProxy<Graphics>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getGraphics({:?})", &this);

        let width = context.jvm().get_field(this.class_instance.as_ref().unwrap(), "w", "I")?;
        let height = context.jvm().get_field(this.class_instance.as_ref().unwrap(), "h", "I")?;

        let instance = context.jvm().instantiate_class("org/kwis/msp/lcdui/Graphics").await?;
        context
            .jvm()
            .invoke_method(
                &instance,
                "org/kwis/msp/lcdui/Graphics",
                "<init>",
                "(Lorg/kwis/msp/lcdui/Image;IIII)V",
                &[
                    JavaValue::Object(Some(this.class_instance.unwrap().clone())),
                    JavaValue::Int(0),
                    JavaValue::Int(0),
                    width,
                    height,
                ],
            )
            .await?;

        Ok(JvmClassInstanceProxy::new(Some(instance)))
    }

    async fn get_width(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getWidth({:?})", &this);

        Ok(context.jvm().get_field(&this.class_instance.unwrap(), "w", "I")?.as_int() as _)
    }

    async fn get_height(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getHeight({:?})", &this);

        Ok(context.jvm().get_field(&this.class_instance.unwrap(), "h", "I")?.as_int() as _)
    }

    pub fn buf(context: &mut dyn JavaContext, this: &ClassInstanceRef) -> JavaResult<Vec<u8>> {
        let java_img_data = context.jvm().get_field(this, "imgData", "[B")?;
        let img_data_len = context.jvm().array_length(java_img_data.as_object_ref().unwrap())?;

        let img_data = context.jvm().load_array(java_img_data.as_object_ref().unwrap(), 0, img_data_len)?;
        let img_data = img_data.into_iter().map(|x| x.as_byte() as u8).collect::<Vec<_>>();

        Ok(img_data)
    }

    pub fn image(context: &mut dyn JavaContext, this: &ClassInstanceRef) -> JavaResult<Box<dyn BackendImage>> {
        Ok(Self::create_canvas(context, this)?.image())
    }

    pub fn canvas<'a>(context: &'a mut dyn JavaContext, this: &'a ClassInstanceRef) -> JavaResult<ImageCanvas<'a>> {
        let canvas = Self::create_canvas(context, this)?;

        Ok(ImageCanvas {
            image: this,
            context,
            canvas,
        })
    }

    fn create_canvas(context: &mut dyn JavaContext, this: &ClassInstanceRef) -> JavaResult<Box<dyn Canvas>> {
        let buf = Self::buf(context, this)?;

        let width = context.jvm().get_field(this, "w", "I")?.as_int();
        let height = context.jvm().get_field(this, "h", "I")?.as_int();
        let bpl = context.jvm().get_field(this, "bpl", "I")?.as_int();

        let bytes_per_pixel = bpl / width;

        let pixel_format = match bytes_per_pixel {
            2 => PixelFormat::Rgb565,
            4 => PixelFormat::Argb,
            _ => panic!("Unsupported pixel format: {}", bytes_per_pixel),
        };

        create_canvas(width as _, height as _, pixel_format, &buf)
    }

    async fn create_image_instance(
        context: &mut dyn JavaContext,
        width: u32,
        height: u32,
        data: &[u8],
        bytes_per_pixel: u32,
    ) -> JavaResult<JvmClassInstanceProxy<Image>> {
        let instance = context.jvm().instantiate_class("org/kwis/msp/lcdui/Image").await?;
        context
            .jvm()
            .invoke_method(&instance, "org/kwis/msp/lcdui/Image", "<init>", "()V", &[])
            .await?;

        let data = data.iter().map(|&x| JavaValue::Byte(x as _)).collect::<Vec<_>>();

        let data_array = context.jvm().instantiate_array("B", data.len() as _).await?;
        context.jvm().store_array(&data_array, 0, &data)?;

        context.jvm().put_field(&instance, "w", "I", JavaValue::Int(width as _))?;
        context.jvm().put_field(&instance, "h", "I", JavaValue::Int(height as _))?;
        context.jvm().put_field(&instance, "imgData", "[B", JavaValue::Object(Some(data_array)))?;
        context
            .jvm()
            .put_field(&instance, "bpl", "I", JavaValue::Int((width * bytes_per_pixel) as _))?;

        Ok(JvmClassInstanceProxy::new(Some(instance)))
    }
}

pub struct ImageCanvas<'a> {
    image: &'a ClassInstanceRef,
    context: &'a mut dyn JavaContext,
    canvas: Box<dyn Canvas>,
}

impl Drop for ImageCanvas<'_> {
    fn drop(&mut self) {
        let data = self.context.jvm().get_field(self.image, "imgData", "[B").unwrap();

        let values = self.canvas.raw().iter().map(|&x| JavaValue::Byte(x as _)).collect::<Vec<_>>();

        self.context.jvm().store_array(data.as_object_ref().unwrap(), 0, &values).unwrap();
    }
}

impl Deref for ImageCanvas<'_> {
    type Target = Box<dyn Canvas>;

    fn deref(&self) -> &Self::Target {
        &self.canvas
    }
}

impl DerefMut for ImageCanvas<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.canvas
    }
}
