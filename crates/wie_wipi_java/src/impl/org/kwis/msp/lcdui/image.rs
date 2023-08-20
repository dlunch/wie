use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::org::kwis::msp::lcdui::Graphics,
};

// class org.kwis.msp.lcdui.Image
pub struct Image {}

impl Image {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new(
                    "createImage",
                    "(II)Lorg/kwis/msp/lcdui/Image;",
                    Self::create_image,
                    JavaMethodAccessFlag::NONE,
                ),
                JavaMethodProto::new(
                    "getGraphics",
                    "()Lorg/kwis/msp/lcdui/Graphics;",
                    Self::get_graphics,
                    JavaMethodAccessFlag::NONE,
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Image>) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Image::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn create_image(context: &mut dyn JavaContext, a0: u32, a1: u32) -> JavaResult<JavaObjectProxy<Image>> {
        log::warn!("stub org.kwis.msp.lcdui.Image::createImage({}, {})", a0, a1);

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Image;")?.cast();
        context.call_method(&instance.cast(), "<init>", "()V", &[]).await?;

        Ok(instance)
    }

    async fn get_graphics(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Graphics>> {
        log::warn!("stub org.kwis.msp.lcdui.Image::get_graphics");

        let instance = context.instantiate("Lorg/kwis/msp/lcdui/Graphics;")?.cast();
        context.call_method(&instance.cast(), "<init>", "(I)V", &[0]).await?;

        Ok(instance)
    }
}
