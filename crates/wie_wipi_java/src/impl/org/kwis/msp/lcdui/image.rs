use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::org::kwis::msp::lcdui::Graphics,
    Array,
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
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Image>) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Image::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn create_image(context: &mut dyn JavaContext, width: u32, height: u32) -> JavaResult<JavaObjectProxy<Image>> {
        log::warn!("stub org.kwis.msp.lcdui.Image::createImage({}, {})", width, height);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;")?.cast();
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn create_image_from_bytes(
        context: &mut dyn JavaContext,
        data: JavaObjectProxy<Array>,
        offset: u32,
        length: u32,
    ) -> JavaResult<JavaObjectProxy<Image>> {
        log::warn!(
            "stub org.kwis.msp.lcdui.Image::createImage({:#x}, {}, {})",
            data.ptr_instance,
            offset,
            length
        );

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;")?.cast();
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn get_graphics(context: &mut dyn JavaContext, this: JavaObjectProxy<Image>) -> JavaResult<JavaObjectProxy<Graphics>> {
        log::warn!("stub org.kwis.msp.lcdui.Image::get_graphics({:#x})", this.ptr_instance);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;")?.cast();
        context
            .call_method(
                &instance.cast(),
                "<init>",
                "(Lorg/kwis/msp/lcdui/Image;IIII)V",
                &[this.ptr_instance, 0, 0, 0, 0],
            )
            .await?;

        Ok(instance)
    }
}
