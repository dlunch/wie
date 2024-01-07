use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    cell::Ref,
    ops::{Deref, DerefMut},
};

use bytemuck::cast_vec;

use java_runtime::java::lang::String;
use java_runtime_base::{Array, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use wie_backend::canvas::{create_canvas, decode_image, ArgbPixel, Canvas, Image as BackendImage, Rgb565Pixel};

use crate::{org::kwis::msp::lcdui::Graphics, JavaClassProto, JavaContextArg};

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

    async fn init(_: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Image>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Image::<init>({:?})", &this);

        Ok(())
    }

    async fn create_image(jvm: &mut Jvm, _: &mut JavaContextArg, width: i32, height: i32) -> JavaResult<JvmClassInstanceHandle<Image>> {
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
    async fn create_image_from_file(
        jvm: &mut Jvm,
        context: &mut JavaContextArg,
        name: JvmClassInstanceHandle<String>,
    ) -> JavaResult<JvmClassInstanceHandle<Image>> {
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
        jvm: &mut Jvm,
        _: &mut JavaContextArg,
        data: JvmClassInstanceHandle<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JavaResult<JvmClassInstanceHandle<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:?}, {}, {})", &data, offset, length);

        let image_data = jvm.load_byte_array(&data, offset as _, length as _)?;
        let image = decode_image(&cast_vec(image_data))?;

        Self::create_image_instance(jvm, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn get_graphics(jvm: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Self>) -> JavaResult<JvmClassInstanceHandle<Graphics>> {
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

    async fn get_width(jvm: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getWidth({:?})", &this);

        jvm.get_field(&this, "w", "I")
    }

    async fn get_height(jvm: &mut Jvm, _: &mut JavaContextArg, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getHeight({:?})", &this);

        jvm.get_field(&this, "h", "I")
    }

    pub fn buf(jvm: &mut Jvm, this: &JvmClassInstanceHandle<Self>) -> JavaResult<Vec<u8>> {
        let java_img_data = jvm.get_field(this, "imgData", "[B")?;
        let img_data_len = jvm.array_length(&java_img_data)?;

        let img_data = jvm.load_byte_array(&java_img_data, 0, img_data_len)?;

        Ok(cast_vec(img_data))
    }

    pub fn image(jvm: &mut Jvm, this: &JvmClassInstanceHandle<Self>) -> JavaResult<Box<dyn BackendImage>> {
        Ok(Self::create_canvas(jvm, this)?.image())
    }

    pub fn canvas<'a>(jvm: &'a mut Jvm, this: &'a JvmClassInstanceHandle<Self>) -> JavaResult<ImageCanvas<'a>> {
        let canvas = Self::create_canvas(jvm, this)?;

        Ok(ImageCanvas { image: this, jvm, canvas })
    }

    fn create_canvas(jvm: &mut Jvm, this: &JvmClassInstanceHandle<Self>) -> JavaResult<Box<dyn Canvas>> {
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

    async fn create_image_instance(
        jvm: &mut Jvm,
        width: u32,
        height: u32,
        data: &[u8],
        bytes_per_pixel: u32,
    ) -> JavaResult<JvmClassInstanceHandle<Image>> {
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
    image: &'a JvmClassInstanceHandle<Image>,
    jvm: &'a mut Jvm,
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
