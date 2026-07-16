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
            methods: vec![
                JavaMethodProto::new(
                    "getSystemProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    Self::get_system_property,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setSystemProperty",
                    "(Ljava/lang/String;Ljava/lang/String;)Z",
                    Self::set_system_property,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn get_system_property(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<String>> {
        let name = JavaLangString::to_rust_string(jvm, &name).await?;
        tracing::warn!("stub org.kwis.msp.handset.HandsetProperty::getSystemProperty({name})");

        let value = match name.as_ref() {
            "VIBRATORLEVEL" => "0",
            _ => "",
        };

        let result = JavaLangString::from_rust_string(jvm, value).await?;
        Ok(result.into())
    }

    async fn set_system_property(_: &Jvm, _: &mut WieJvmContext, id: ClassInstanceRef<String>, value: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.handset.HandsetProperty::setSystemProperty({id:?}, {value:?})");

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn test_set_system_property_returns_false() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let id: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "storage.test").await?.into();
            let value: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "value").await?.into();
            let result: bool = jvm
                .invoke_static(
                    "org/kwis/msp/handset/HandsetProperty",
                    "setSystemProperty",
                    "(Ljava/lang/String;Ljava/lang/String;)Z",
                    (id, value),
                )
                .await?;

            assert!(!result);
            Ok(())
        })
    }
}
