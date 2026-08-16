use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msf.io.URL
pub struct URL;

impl URL {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/io/URL",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "find",
                    "(Ljava/lang/String;)Lorg/kwis/msf/io/Socket;",
                    Self::find,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn find(jvm: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Self>> {
        Err(jvm.exception("org/kwis/msf/io/SchemeNotFoundException", "Network is not supported").await)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    use super::URL;

    #[test]
    fn find_throws_scheme_not_found_exception() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let _: ClassInstanceRef<URL> = jvm.new_class("org/kwis/msf/io/URL", "()V", ()).await?.into();
            let url: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "http://example.com").await?.into();
            let result: JvmResult<ClassInstanceRef<URL>> = jvm
                .invoke_static("org/kwis/msf/io/URL", "find", "(Ljava/lang/String;)Lorg/kwis/msf/io/Socket;", (url,))
                .await;

            let Err(JavaError::JavaException(exception)) = result else {
                panic!("URL.find returned without throwing SchemeNotFoundException");
            };
            assert!(jvm.is_instance(&*exception, "org/kwis/msf/io/SchemeNotFoundException"));
            assert!(jvm.is_instance(&*exception, "java/io/IOException"));

            Ok(())
        })
    }
}
