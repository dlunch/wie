use alloc::{boxed::Box, vec};

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    method::MethodBody,
    proxy::JavaObjectProxy,
    JavaError,
};

// class org.kwis.msp.lcdui.Card
pub struct Card {}

impl Card {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init),
                JavaMethodProto::new("<init>", "(I)V", Self::init_1),
                JavaMethodProto::new("getWidth", "()I", Self::get_width),
                JavaMethodProto::new("getHeight", "()I", Self::get_height),
            ],
            fields: vec![],
        }
    }

    async fn init(context: &mut dyn JavaContext, instance: JavaObjectProxy) -> JavaResult<()> {
        log::debug!("Card::<init>({:#x})", instance.ptr_instance);

        context.spawn(Box::new(CardLoop { instance }))?;

        Ok(())
    }

    async fn init_1(_: &mut dyn JavaContext, instance: JavaObjectProxy, a0: u32) -> JavaResult<()> {
        log::debug!("Card::<init>({:#x}, {})", instance.ptr_instance, a0);

        Ok(())
    }

    async fn get_width(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::debug!("Card::get_width");

        Ok(320) // TODO: hardcoded
    }

    async fn get_height(_: &mut dyn JavaContext) -> JavaResult<u32> {
        log::debug!("Card::get_height");

        Ok(480) // TODO: hardcoded
    }
}

struct CardLoop {
    instance: JavaObjectProxy,
}

#[async_trait::async_trait(?Send)]
impl MethodBody<JavaError> for CardLoop {
    async fn call(&self, context: &mut dyn JavaContext, _: &[u32]) -> Result<u32, JavaError> {
        loop {
            context.sleep(16).await;

            context
                .call_method(&self.instance, "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", &[0])
                .await
                .unwrap();
        }

        #[allow(unreachable_code)]
        Ok(0)
    }
}
