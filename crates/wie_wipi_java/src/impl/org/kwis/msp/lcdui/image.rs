use alloc::{vec, vec::Vec};
use core::ops::{Deref, DerefMut};

use bytemuck::cast_slice;

use wie_backend::{Canvas, CanvasMut};

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
        log::debug!("org.kwis.msp.lcdui.Image::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn create_image(context: &mut dyn JavaContext, width: u32, height: u32) -> JavaResult<JavaObjectProxy<Image>> {
        log::debug!("org.kwis.msp.lcdui.Image::createImage({}, {})", width, height);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;")?;
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

        let data = context.instantiate_array("B", width * height * 4)?;

        context.put_field(&instance, "w", width)?;
        context.put_field(&instance, "h", height)?;
        context.put_field(&instance, "imgData", data.ptr_instance)?;
        context.put_field(&instance, "bpl", width * 4)?;

        Ok(instance.cast())
    }

    async fn create_image_from_bytes(
        context: &mut dyn JavaContext,
        data: JavaObjectProxy<Array>,
        offset: u32,
        length: u32,
    ) -> JavaResult<JavaObjectProxy<Image>> {
        log::debug!("org.kwis.msp.lcdui.Image::createImage({:#x}, {}, {})", data.ptr_instance, offset, length);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;")?;
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

        let image_data = context.load_array_u8(&data, offset, length)?;
        let canvas = Canvas::from_image(&image_data)?;

        let data = context.instantiate_array("B", canvas.width() * canvas.height() * 4)?;
        let buffer = cast_slice(canvas.buffer());
        context.store_array_u8(&data, 0, buffer)?;

        context.put_field(&instance, "w", canvas.width())?;
        context.put_field(&instance, "h", canvas.height())?;
        context.put_field(&instance, "imgData", data.ptr_instance)?;
        context.put_field(&instance, "bpl", canvas.width() * 4)?;

        Ok(instance.cast())
    }

    async fn get_graphics(context: &mut dyn JavaContext, this: JavaObjectProxy<Image>) -> JavaResult<JavaObjectProxy<Graphics>> {
        log::debug!("org.kwis.msp.lcdui.Image::getGraphics({:#x})", this.ptr_instance);

        let width = context.get_field(&this.cast(), "w")?;
        let height = context.get_field(&this.cast(), "h")?;

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;")?.cast();
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

    pub fn get_buf(context: &dyn JavaContext, this: &JavaObjectProxy<Image>) -> JavaResult<Vec<u8>> {
        let java_img_data = JavaObjectProxy::new(context.get_field(&this.cast(), "imgData")?);
        let img_data_len = context.array_length(&java_img_data)?;

        let img_data = context.load_array_u8(&java_img_data.cast(), 0, img_data_len)?;

        Ok(img_data)
    }

    pub fn get_canvas(context: &dyn JavaContext, this: &JavaObjectProxy<Image>) -> JavaResult<Canvas> {
        let buf = Self::get_buf(context, this)?;

        let width = context.get_field(&this.cast(), "w")?;
        let height = context.get_field(&this.cast(), "h")?;

        let canvas = Canvas::from_raw(width, height, buf.to_vec());

        Ok(canvas)
    }

    pub fn get_canvas_mut<'a>(context: &'a mut dyn JavaContext, this: &'a JavaObjectProxy<Image>) -> JavaResult<ImageCanvas<'a>> {
        let buf = Self::get_buf(context, this)?;

        let width = context.get_field(&this.cast(), "w")?;
        let height = context.get_field(&this.cast(), "h")?;

        let canvas = CanvasMut::from_raw(width, height, buf.to_vec());

        Ok(ImageCanvas {
            image: this,
            context,
            canvas,
        })
    }
}

pub struct ImageCanvas<'a> {
    image: &'a JavaObjectProxy<Image>,
    context: &'a mut dyn JavaContext,
    canvas: CanvasMut,
}

impl Drop for ImageCanvas<'_> {
    fn drop(&mut self) {
        let data = JavaObjectProxy::new(self.context.get_field(&self.image.cast(), "imgData").unwrap());

        let buffer: &[u8] = cast_slice(self.canvas.buffer());
        self.context.store_array_u8(&data, 0, buffer).unwrap();
    }
}

impl Deref for ImageCanvas<'_> {
    type Target = CanvasMut;

    fn deref(&self) -> &Self::Target {
        &self.canvas
    }
}

impl DerefMut for ImageCanvas<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.canvas
    }
}
