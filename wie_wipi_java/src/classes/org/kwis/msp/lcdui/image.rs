use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    cell::Ref,
    ops::{Deref, DerefMut},
};

use bytemuck::cast_vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto, JavaResult};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm};

use wie_backend::canvas::{create_canvas, decode_image, ArgbPixel, Canvas, Image as BackendImage, Rgb565Pixel};

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

    async fn init(_: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Image>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Image::<init>({:?})", &this);

        Ok(())
    }

    async fn create_image(jvm: &Jvm, _: &mut WIPIJavaContext, width: i32, height: i32) -> JavaResult<ClassInstanceRef<Image>> {
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

    #[allow(clippy::await_holding_refcell_ref)] // We manually drop Ref
    async fn create_image_from_file(jvm: &Jvm, context: &mut WIPIJavaContext, name: ClassInstanceRef<String>) -> JavaResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?})", &name);

        let name = String::to_rust_string(jvm, &name)?;
        let normalized_name = if let Some(x) = name.strip_prefix('/') { x } else { &name };

        let id = context.system().resource().id(normalized_name).unwrap();
        let system_clone = context.system().clone();
        let image_data = Ref::map(system_clone.resource(), |x| x.data(id));

        let image = decode_image(&image_data)?;
        drop(image_data);

        Self::create_image_instance(jvm, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn create_image_from_bytes(
        jvm: &Jvm,
        _: &mut WIPIJavaContext,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JavaResult<ClassInstanceRef<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?}, {}, {})", &data, offset, length);

        let image_data = jvm.load_byte_array(&data, offset as _, length as _)?;
        let image = decode_image(&cast_vec(image_data))?;

        Self::create_image_instance(jvm, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn get_graphics(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getGraphics({:?})", &this);

        let width: i32 = jvm.get_field(&this, "w", "I")?;
        let height: i32 = jvm.get_field(&this, "h", "I")?;

        let instance = jvm
            .new_class(
                "org/kwis/msp/lcdui/Graphics",
                "(Lorg/kwis/msp/lcdui/Image;IIII)V",
                (this.clone(), 0, 0, width, height),
            )
            .await?;

        Ok(instance.into())
    }

    async fn get_width(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getWidth({:?})", &this);

        jvm.get_field(&this, "w", "I")
    }

    async fn get_height(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getHeight({:?})", &this);

        jvm.get_field(&this, "h", "I")
    }

    pub fn buf(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JavaResult<Vec<u8>> {
        let java_img_data = jvm.get_field(this, "imgData", "[B")?;
        let img_data_len = jvm.array_length(&java_img_data)?;

        let img_data = jvm.load_byte_array(&java_img_data, 0, img_data_len)?;

        Ok(cast_vec(img_data))
    }

    pub fn image(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JavaResult<Box<dyn BackendImage>> {
        Ok(Self::create_canvas(jvm, this)?.image())
    }

    pub fn canvas<'a>(jvm: &'a Jvm, this: &'a ClassInstanceRef<Self>) -> JavaResult<ImageCanvas<'a>> {
        let canvas = Self::create_canvas(jvm, this)?;

        Ok(ImageCanvas { image: this, jvm, canvas })
    }

    fn create_canvas(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JavaResult<Box<dyn Canvas>> {
        let buf = Self::buf(jvm, this)?;

        let width: i32 = jvm.get_field(this, "w", "I")?;
        let height: i32 = jvm.get_field(this, "h", "I")?;
        let bpl: i32 = jvm.get_field(this, "bpl", "I")?;

        let bytes_per_pixel = bpl / width;

        match bytes_per_pixel {
            2 => create_canvas::<Rgb565Pixel>(width as _, height as _, &buf),
            4 => create_canvas::<ArgbPixel>(width as _, height as _, &buf),
            _ => panic!("Unsupported pixel format: {}", bytes_per_pixel),
        }
    }

    async fn create_image_instance(jvm: &Jvm, width: u32, height: u32, data: &[u8], bytes_per_pixel: u32) -> JavaResult<ClassInstanceRef<Image>> {
        let mut instance = jvm.new_class("org/kwis/msp/lcdui/Image", "()V", []).await?;

        let mut data_array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.store_byte_array(&mut data_array, 0, cast_vec(data.to_vec()))?;

        jvm.put_field(&mut instance, "w", "I", width as i32)?;
        jvm.put_field(&mut instance, "h", "I", height as i32)?;
        jvm.put_field(&mut instance, "imgData", "[B", data_array)?;
        jvm.put_field(&mut instance, "bpl", "I", (width * bytes_per_pixel) as i32)?;

        Ok(instance.into())
    }
}

pub struct ImageCanvas<'a> {
    image: &'a ClassInstanceRef<Image>,
    jvm: &'a Jvm,
    canvas: Box<dyn Canvas>,
}

impl Drop for ImageCanvas<'_> {
    fn drop(&mut self) {
        let mut data = self.jvm.get_field(self.image, "imgData", "[B").unwrap();

        self.jvm.store_byte_array(&mut data, 0, cast_vec(self.canvas.raw().to_vec())).unwrap();
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
