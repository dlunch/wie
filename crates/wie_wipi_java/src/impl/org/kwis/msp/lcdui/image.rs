use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Image
pub struct Image {}

impl Image {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("createImage", "(II)Lorg/kwis/msp/lcdui/Image;", Self::create_image),
                JavaMethodProto::new("getGraphics", "()Lorg/kwis/msp/lcdui/Graphics;", Self::get_graphics),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Image::<init>({:#x})", instance.ptr_instance);

        Ok(())
    }

    async fn create_image(context: &mut dyn JavaContext, a0: u32, a1: u32) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Image::createImage({}, {})", a0, a1);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;")?;
        context.call_method(&instance, "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn get_graphics(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy> {
        log::warn!("stub Image::get_graphics");

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;")?;
        context.call_method(&instance, "<init>", "()V", &[]).await?;

        Ok(instance)
    }
}
