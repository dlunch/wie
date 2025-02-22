use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.handset.HandsetProperty
pub struct HandsetProperty;

impl HandsetProperty {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/handset/HandsetProperty",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "getSystemProperty",
                "(Ljava/lang/String;)Ljava/lang/String;",
                Self::get_system_property,
                MethodAccessFlags::STATIC,
            )],
            fields: vec![],
        }
    }

    async fn get_system_property(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<String>> {
        let name = JavaLangString::to_rust_string(jvm, &name).await?;
        tracing::warn!("stub org.kwis.msp.handset.HandsetProperty::getSystemProperty({})", name);

        let value = match name.as_ref() {
            "VIBRATORLEVEL" => "0",
            _ => "",
        };

        let result = JavaLangString::from_rust_string(jvm, value).await?;
        Ok(result.into())
    }
}
