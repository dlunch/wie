use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface javax.microedition.io.Connection
#[java_class(name = "javax/microedition/io/Connection", public, interface, abstract)]
pub struct Connection;

#[java_methods]
impl Connection {
    #[method(public)]
    async fn close(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;
}
