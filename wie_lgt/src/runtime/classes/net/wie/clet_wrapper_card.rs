use alloc::vec;

use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use super::CletWrapperContext;

// class net.wie.CletWrapperCard
pub struct CletWrapperCard;

impl CletWrapperCard {
    pub fn as_proto() -> JavaClassProto<CletWrapperContext> {
        JavaClassProto {
            name: "net/wie/CletWrapperCard",
            parent_class: Some("org/kwis/msp/lcdui/Card"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(II)V", Self::init, Default::default()),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("paintClet", "I", Default::default()),
                JavaFieldProto::new("handleInput", "I", Default::default()),
            ],
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut CletWrapperContext,
        mut this: ClassInstanceRef<Self>,
        paint_clet: i32,
        handle_input: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapperCard::<init>({:?}, {:#x}, {:#x})", &this, paint_clet, handle_input);

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "paintClet", "I", paint_clet).await?;
        jvm.put_field(&mut this, "handleInput", "I", handle_input).await?;

        Ok(())
    }

    async fn paint(jvm: &Jvm, context: &mut CletWrapperContext, this: ClassInstanceRef<Self>, _graphics: ClassInstanceRef<()>) -> JvmResult<()> {
        tracing::debug!("net.wie.CletWrapperCard::paint({:?})", &this);

        let paint_clet: i32 = jvm.get_field(&this, "paintClet", "I").await?;
        let _: () = context.core.run_function(paint_clet as _, &[]).await.unwrap();

        Ok(())
    }
}
