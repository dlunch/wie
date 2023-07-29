use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    from_java_string,
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.handset.HandsetProperty
pub struct HandsetProperty {}

impl HandsetProperty {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new(
                "getSystemProperty",
                "(Ljava/lang/String;)Ljava/lang/String;",
                Self::get_system_property,
            )],
            fields: vec![],
        }
    }

    async fn get_system_property(context: &mut dyn JavaContext, name: JavaObjectProxy) -> JavaResult<JavaObjectProxy> {
        let name = from_java_string(context, &name)?;

        log::warn!("stub org.kwis.msp.handset.HandsetProperty::getSystemProperty({})", name);

        Ok(JavaObjectProxy::new(0))
    }
}
