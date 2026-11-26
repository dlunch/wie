use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.handset.HandsetProperty
#[java_class(name = "org/kwis/msp/handset/HandsetProperty", public)]
pub struct HandsetProperty;

#[java_methods]
impl HandsetProperty {
    #[method(public, static)]
    async fn get_system_property(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<String>> {
        let name = JavaLangString::to_rust_string(jvm, &name).await?;
        tracing::debug!("org.kwis.msp.handset.HandsetProperty::getSystemProperty({name})");

        let value = match name.as_ref() {
            "VIBRATORLEVEL" => "0",
            "VOLUMELEVEL" => "100",
            "PHONENUMBER" => {
                let key = JavaLangString::from_rust_string(jvm, "wie.phoneNumber").await?;
                return jvm
                    .invoke_static("java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;", (key,))
                    .await;
            }
            _ => "",
        };

        let result = JavaLangString::from_rust_string(jvm, value).await?;
        Ok(result.into())
    }

    #[method(public, static)]
    async fn set_system_property(_: &Jvm, _: &mut WieJvmContext, id: ClassInstanceRef<String>, value: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.handset.HandsetProperty::setSystemProperty({id:?}, {value:?})");

        Ok(false)
    }
}
