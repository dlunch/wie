use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    from_java_string,
    proxy::JavaObjectProxy,
    r#impl::java::lang::String,
    to_java_string,
};

// class org.kwis.msp.handset.HandsetProperty
pub struct HandsetProperty {}

impl HandsetProperty {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "getSystemProperty",
                "(Ljava/lang/String;)Ljava/lang/String;",
                Self::get_system_property,
                JavaMethodAccessFlag::NONE,
            )],
            fields: vec![],
        }
    }

    async fn get_system_property(context: &mut dyn JavaContext, name: JavaObjectProxy<String>) -> JavaResult<JavaObjectProxy<String>> {
        let name = from_java_string(context, &name)?;
        log::warn!("stub org.kwis.msp.handset.HandsetProperty::getSystemProperty({})", name);

        let result = to_java_string(context, "").await?;
        Ok(result)
    }
}
