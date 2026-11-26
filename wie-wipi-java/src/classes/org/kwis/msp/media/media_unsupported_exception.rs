use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.media.MediaUnsupportedException
#[java_class(name = "org/kwis/msp/media/MediaUnsupportedException", extends = "java/lang/Exception", public)]
pub struct MediaUnsupportedException;

#[java_methods]
impl MediaUnsupportedException {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.MediaUnsupportedException::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Exception", "<init>", "()V", ()).await
    }

    #[method(constructor, public)]
    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.MediaUnsupportedException::<init>({this:?}, {message:?})");

        jvm.invoke_special(&this, "java/lang/Exception", "<init>", "(Ljava/lang/String;)V", (message,))
            .await
    }
}
