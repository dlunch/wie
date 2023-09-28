use alloc::{boxed::Box, vec, vec::Vec};
use core::ops::{Deref, DerefMut};

use bytemuck::{cast_slice, cast_vec};

use wie_backend::canvas::{create_canvas, decode_image, Canvas, Image as BackendImage};

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::org::kwis::msp::lcdui::Graphics,
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
                    "([BII)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image_from_bytes,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new("getGraphics", "()Lorg/kwis/msp/lcdui/Graphics;", Self::get_graphics, JavaMethodFlag::NONE),
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

        let data = context.instantiate_array("B", (width * height * bytes_per_pixel) as _).await?;

        context.put_field(&instance, "w", width as _)?;
        context.put_field(&instance, "h", height as _)?;
        context.put_field(&instance, "imgData", data.ptr_instance)?;
        context.put_field(&instance, "bpl", (width * bytes_per_pixel) as _)?;

        Ok(instance.cast())
    }

    async fn create_image_from_bytes(
        context: &mut dyn JavaContext,
        data: JavaObjectProxy<Array>,
        offset: i32,
        length: i32,
    ) -> JavaResult<JavaObjectProxy<Image>> {
        tracing::debug!("org.kwis.msp.lcdui.Image::createImage({:#x}, {}, {})", data.ptr_instance, offset, length);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;").await?;
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

        let image_data = context.load_array_i8(&data, offset as _, length as _)?;
        let image = decode_image(cast_slice(&image_data))?;

        let bytes_per_pixel = image.bytes_per_pixel();

        let data = context
            .instantiate_array("B", (image.width() * image.height() * bytes_per_pixel) as _)
            .await?;
        context.store_array_i8(&data, 0, cast_slice(image.raw()))?;

        context.put_field(&instance, "w", image.width() as _)?;
        context.put_field(&instance, "h", image.height() as _)?;
        context.put_field(&instance, "imgData", data.ptr_instance)?;
        context.put_field(&instance, "bpl", (image.width() * bytes_per_pixel) as _)?;

        Ok(instance.cast())
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

        create_canvas(width as _, height as _, (bytes_per_pixel * 8) as _, &buf)
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
