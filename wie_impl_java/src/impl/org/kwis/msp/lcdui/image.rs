use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    cell::Ref,
    ops::{Deref, DerefMut},
};

use bytemuck::{cast_slice, cast_vec};

use wie_backend::canvas::{create_canvas, decode_image, Canvas, Image as BackendImage, PixelFormat};

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::{java::lang::String, org::kwis::msp::lcdui::Graphics},
    Array, JavaFieldAccessFlag,
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
                JavaMethodProto::new("createImage", "(II)Lorg/kwis/msp/lcdui/Image;", Self::create_image, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_file,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "([BII)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_bytes,
                    JavaMethodFlag::NONE,
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

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Image>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Image::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn create_image(context: &mut dyn JavaContext, width: i32, height: i32) -> JavaResult<JavaObjectProxy<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({}, {})", width, height);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;").await?;
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

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
    async fn create_image_from_file(context: &mut dyn JavaContext, name: JavaObjectProxy<String>) -> JavaResult<JavaObjectProxy<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:#x})", name.ptr_instance);

        let name = String::to_rust_string(context, &name)?;
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
        data: JavaObjectProxy<Array>,
        offset: i32,
        length: i32,
    ) -> JavaResult<JavaObjectProxy<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:#x}, {}, {})", data.ptr_instance, offset, length);

        let image_data = context.load_array_i8(&data, offset as _, length as _)?;
        let image = decode_image(cast_slice(&image_data))?;

        Self::create_image_instance(context, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn get_graphics(context: &mut dyn JavaContext, this: JavaObjectProxy<Image>) -> JavaResult<JavaObjectProxy<Graphics>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getGraphics({:#x})", this.ptr_instance);

        let width = context.get_field(&this.cast(), "w")?;
        let height = context.get_field(&this.cast(), "h")?;

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;").await?.cast();
        context
            .call_method(
                &instance.cast(),
                "<init>",
                "(Lorg/kwis/msp/lcdui/Image;IIII)V",
                &[this.ptr_instance, 0, 0, width, height],
            )
            .await?;

        Ok(instance)
    }

    async fn get_width(context: &mut dyn JavaContext, this: JavaObjectProxy<Image>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getWidth({:#x})", this.ptr_instance);

        Ok(context.get_field(&this.cast(), "w")? as _)
    }

    async fn get_height(context: &mut dyn JavaContext, this: JavaObjectProxy<Image>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.Image::getHeight({:#x})", this.ptr_instance);

        Ok(context.get_field(&this.cast(), "h")? as _)
    }

    pub fn buf(context: &dyn JavaContext, this: &JavaObjectProxy<Image>) -> JavaResult<Vec<u8>> {
        let java_img_data = JavaObjectProxy::new(context.get_field(&this.cast(), "imgData")?);
        let img_data_len = context.array_length(&java_img_data)?;

        let img_data = context.load_array_i8(&java_img_data.cast(), 0, img_data_len)?;

        Ok(cast_vec(img_data))
    }

    pub fn image(context: &dyn JavaContext, this: &JavaObjectProxy<Image>) -> JavaResult<Box<dyn BackendImage>> {
        Ok(Self::create_canvas(context, this)?.image())
    }

    pub fn canvas<'a>(context: &'a mut dyn JavaContext, this: &'a JavaObjectProxy<Image>) -> JavaResult<ImageCanvas<'a>> {
        let canvas = Self::create_canvas(context, this)?;

        Ok(ImageCanvas {
            image: this,
            context,
            canvas,
        })
    }

    fn create_canvas(context: &dyn JavaContext, this: &JavaObjectProxy<Image>) -> JavaResult<Box<dyn Canvas>> {
        let buf = Self::buf(context, this)?;

        let width = context.get_field(&this.cast(), "w")?;
        let height = context.get_field(&this.cast(), "h")?;
        let bpl = context.get_field(&this.cast(), "bpl")?;

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
    ) -> JavaResult<JavaObjectProxy<Image>> {
        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;").await?;
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

        let data_array = context.instantiate_array("B", data.len() as _).await?;
        context.store_array_i8(&data_array, 0, cast_slice(data))?;

        context.put_field(&instance, "w", width as _)?;
        context.put_field(&instance, "h", height as _)?;
        context.put_field(&instance, "imgData", data_array.ptr_instance)?;
        context.put_field(&instance, "bpl", (width * bytes_per_pixel) as _)?;

        Ok(instance.cast())
    }
}

pub struct ImageCanvas<'a> {
    image: &'a JavaObjectProxy<Image>,
    context: &'a mut dyn JavaContext,
    canvas: Box<dyn Canvas>,
}

impl Drop for ImageCanvas<'_> {
    fn drop(&mut self) {
        let data = JavaObjectProxy::new(self.context.get_field(&self.image.cast(), "imgData").unwrap());

        self.context.store_array_i8(&data, 0, cast_slice(self.canvas.raw())).unwrap();
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
