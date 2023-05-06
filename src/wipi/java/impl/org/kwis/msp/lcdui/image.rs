use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaObjectProxy, JavaResult};

// class org.kwis.msp.lcdui.Image
pub struct Image {}

impl Image {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init),
                JavaMethodProto::new("createImage", "(II)Lorg/kwis/msp/lcdui/Image;", Self::create_image),
                JavaMethodProto::new("getGraphics", "()Lorg/kwis/msp/lcdui/Graphics;", Self::get_graphics),
            ],
        }
    }

    fn init(_: &mut JavaContext, _: u32) -> JavaResult<()> {
        log::debug!("Image::<init>");

        Ok(())
    }

    fn create_image(context: &mut JavaContext, a0: u32, a1: u32) -> JavaResult<JavaObjectProxy> {
        log::debug!("Image::createImage({}, {})", a0, a1);

        context.instantiate("Lorg/kwis/msp/lcdui/Image;")
    }

    fn get_graphics(context: &mut JavaContext) -> JavaResult<JavaObjectProxy> {
        log::debug!("Image::get_graphics");

        context.instantiate("Lorg/kwis/msp/lcdui/Graphics;")
    }
}
