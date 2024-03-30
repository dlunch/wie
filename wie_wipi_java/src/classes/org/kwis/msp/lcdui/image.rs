use alloc::{boxed::Box, vec, vec::Vec};
use core::ops::{Deref, DerefMut};

use bytemuck::{cast_vec, pod_collect_to_vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_backend::canvas::{decode_image, ArgbPixel, Canvas, Image as BackendImage, ImageBufferCanvas, Rgb565Pixel, VecImageBuffer};

use crate::{
    classes::org::kwis::msp::lcdui::Graphics,
    context::{WIPIJavaClassProto, WIPIJavaContext},
};

// class org.kwis.msp.lcdui.Image
pub struct Image {}

impl Image {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "createImage",
                    "(II)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_file,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "([BII)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_bytes,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getGraphics", "()Lorg/kwis/msp/lcdui/Graphics;", Self::get_graphics, Default::default()),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("w", "I", Default::default()),
                JavaFieldProto::new("h", "I", Default::default()),
                JavaFieldProto::new("imgData", "[B", Default::default()),
                JavaFieldProto::new("bpl", "I", Default::default()),
            ],
        }
    }

    async fn init(_: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Image::<init>({:?})", &this);

        Ok(())
    }

    async fn create_image(jvm: &Jvm, _: &mut WIPIJavaContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({}, {})", width, height);

        let bytes_per_pixel = 4;

        Self::create_image_instance(
            jvm,
            width as _,
            height as _,
            &vec![0; (width * height * bytes_per_pixel) as usize],
            bytes_per_pixel as _,
        )
        .await
    }

    async fn create_image_from_file(jvm: &Jvm, context: &mut WIPIJavaContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?})", &name);

        let name = JavaLangString::to_rust_string(jvm, &name).await?;
        let normalized_name = if let Some(x) = name.strip_prefix('/') { x } else { &name };

        let id = context.system().resource().id(normalized_name).unwrap();
        let system_clone = context.system().clone();

        let image = {
            let resource = system_clone.resource();
            let image_data = resource.data(id);

            decode_image(image_data)
        }
        .unwrap();

        Self::create_image_instance(jvm, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn create_image_from_bytes(
        jvm: &Jvm,
        _: &mut WIPIJavaContext,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?}, {}, {})", &data, offset, length);

        let image_data = jvm.load_byte_array(&data, offset as _, length as _).await?;
        let image = decode_image(&cast_vec(image_data)).unwrap();

        Self::create_image_instance(jvm, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn get_graphics(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getGraphics({:?})", &this);

        let width: i32 = jvm.get_field(&this, "w", "I").await?;
        let height: i32 = jvm.get_field(&this, "h", "I").await?;

        let instance = jvm
            .new_class(
                "org/kwis/msp/lcdui/Graphics",
                "(Lorg/kwis/msp/lcdui/Image;IIII)V",
                (this.clone(), 0, 0, width, height),
            )
            .await?;

        Ok(instance.into())
    }

    async fn get_width(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getWidth({:?})", &this);

        jvm.get_field(&this, "w", "I").await
    }

    async fn get_height(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getHeight({:?})", &this);

        jvm.get_field(&this, "h", "I").await
    }

    pub async fn buf(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Vec<u8>> {
        let java_img_data = jvm.get_field(this, "imgData", "[B").await?;
        let img_data_len = jvm.array_length(&java_img_data).await?;

        let img_data = jvm.load_byte_array(&java_img_data, 0, img_data_len).await?;

        Ok(cast_vec(img_data))
    }

    pub async fn image(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn BackendImage>> {
        let buf = Self::buf(jvm, this).await?;

        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let height: i32 = jvm.get_field(this, "h", "I").await?;
        let bpl: i32 = jvm.get_field(this, "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(match bytes_per_pixel {
            2 => Box::new(VecImageBuffer::<Rgb565Pixel>::from_raw(width as _, height as _, pod_collect_to_vec(&buf))) as Box<_>,
            4 => Box::new(VecImageBuffer::<ArgbPixel>::from_raw(width as _, height as _, pod_collect_to_vec(&buf))) as Box<_>,
            _ => unimplemented!("Unsupported pixel format: {}", bytes_per_pixel),
        })
    }

    pub async fn canvas<'a>(jvm: &'a Jvm, this: &'a ClassInstanceRef<Self>) -> JvmResult<ImageCanvas<'a>> {
        let buf = Self::buf(jvm, this).await?;

        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let height: i32 = jvm.get_field(this, "h", "I").await?;
        let bpl: i32 = jvm.get_field(this, "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(ImageCanvas::new(jvm, this, width as _, height as _, bytes_per_pixel as _, buf))
    }

    async fn create_image_instance(jvm: &Jvm, width: u32, height: u32, data: &[u8], bytes_per_pixel: u32) -> JvmResult<ClassInstanceRef<Image>> {
        let mut instance = jvm.new_class("org/kwis/msp/lcdui/Image", "()V", []).await?;

        let mut data_array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.store_byte_array(&mut data_array, 0, cast_vec(data.to_vec())).await?;

        jvm.put_field(&mut instance, "w", "I", width as i32).await?;
        jvm.put_field(&mut instance, "h", "I", height as i32).await?;
        jvm.put_field(&mut instance, "imgData", "[B", data_array).await?;
        jvm.put_field(&mut instance, "bpl", "I", (width * bytes_per_pixel) as i32).await?;

        Ok(instance.into())
    }
}

pub struct ImageCanvas<'a> {
    image: &'a ClassInstanceRef<Image>,
    jvm: &'a Jvm,
    canvas: Box<dyn Canvas>,
    flushed: bool,
}

impl<'a> ImageCanvas<'a> {
    pub fn new(jvm: &'a Jvm, image: &'a ClassInstanceRef<Image>, width: u32, height: i32, bytes_per_pixel: u32, buf: Vec<u8>) -> Self {
        let canvas: Box<dyn Canvas> = match bytes_per_pixel {
            2 => Box::new(ImageBufferCanvas::new(VecImageBuffer::<Rgb565Pixel>::from_raw(
                width as _,
                height as _,
                pod_collect_to_vec(&buf),
            ))),
            4 => Box::new(ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::from_raw(
                width as _,
                height as _,
                pod_collect_to_vec(&buf),
            ))),
            _ => unimplemented!("Unsupported pixel format: {}", bytes_per_pixel),
        };

        Self {
            image,
            jvm,
            canvas,
            flushed: false,
        }
    }

    // We don't have async drop yet..
    pub async fn flush(mut self) {
        let mut data = self.jvm.get_field(self.image, "imgData", "[B").await.unwrap();

        self.jvm
            .store_byte_array(&mut data, 0, cast_vec(self.canvas.image().raw().to_vec()))
            .await
            .unwrap();
        self.flushed = true
    }
}

impl Drop for ImageCanvas<'_> {
    fn drop(&mut self) {
        if !self.flushed {
            panic!("ImageCanvas was dropped without flushing")
        }
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
