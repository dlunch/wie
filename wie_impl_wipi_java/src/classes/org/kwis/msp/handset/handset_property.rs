use alloc::vec;

use java_runtime::classes::java::lang::String;
use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use crate::{WIPIJavaClassProto, WIPIJavaContxt};

// class org.kwis.msp.handset.HandsetProperty
pub struct HandsetProperty {}

impl HandsetProperty {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "getSystemProperty",
                "(Ljava/lang/String;)Ljava/lang/String;",
                Self::get_system_property,
                JavaMethodFlag::STATIC,
            )],
            fields: vec![],
        }
    }

    async fn get_system_property(
        jvm: &mut Jvm,
        _: &mut WIPIJavaContxt,
        name: JvmClassInstanceHandle<String>,
    ) -> JavaResult<JvmClassInstanceHandle<String>> {
        let name = String::to_rust_string(jvm, &name)?;
        tracing::warn!("stub org.kwis.msp.handset.HandsetProperty::getSystemProperty({})", name);

        let result = String::from_rust_string(jvm, "").await?;
        Ok(result)
    }
}
