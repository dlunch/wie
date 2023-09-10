use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    JavaFieldAccessFlag, JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![JavaFieldProto::new("dis", "Lorg/kwis/msp/lcdui/Display;", JavaFieldAccessFlag::STATIC)],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<Jlet>) -> JavaResult<()> {
        tracing::warn!("org.kwis.msp.lcdui.Jlet::<init>");

        let display = context.instantiate("Lorg/kwis/msp/lcdui/Display;")?;
        context
            .call_method(
                &display.cast(),
                "<init>",
                "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                &[this.ptr_instance, 0],
            )
            .await?;

        context.put_static_field("org/kwis/msp/lcdui/Jlet", "dis", display.ptr_instance)?;

        Ok(())
    }
}
