use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::org::kwis::msf::io::Socket;

// class org.kwis.msf.io.URL
#[java_class(name = "org/kwis/msf/io/URL", public)]
pub struct URL;

#[java_methods]
impl URL {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[method(public, static)]
    async fn find(jvm: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Socket>> {
        Err(jvm.exception("org/kwis/msf/io/SchemeNotFoundException", "Network is not supported").await)
    }
}
