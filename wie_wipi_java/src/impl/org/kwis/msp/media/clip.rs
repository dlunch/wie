use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
    r#impl::java::lang::String,
};

// class org.kwis.msp.media.Clip
pub struct Clip {}

impl Clip {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;Ljava/lang/String;)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("setVolume", "(I)Z", Self::set_volume, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(
        _: &mut dyn JavaContext,
        this: JavaObjectProxy<Clip>,
        r#type: JavaObjectProxy<String>,
        resource_name: JavaObjectProxy<String>,
    ) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.media.Clip::<init>({:#x}, {:#x}, {:#x})",
            this.ptr_instance,
            r#type.ptr_instance,
            resource_name.ptr_instance
        );

        Ok(())
    }

    async fn set_volume(_: &mut dyn JavaContext, this: JavaObjectProxy<Clip>, level: u32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setVolume({:#x}, {})", this.ptr_instance, level);

        Ok(())
    }
}
