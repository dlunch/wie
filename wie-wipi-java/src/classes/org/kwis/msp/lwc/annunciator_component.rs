use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.lwc.AnnunciatorComponent
#[java_class(name = "org/kwis/msp/lwc/AnnunciatorComponent", extends = "org/kwis/msp/lwc/ShellComponent", public)]
pub struct AnnunciatorComponent;

#[java_methods]
impl AnnunciatorComponent {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<AnnunciatorComponent>, a0: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::<init>({this:?}, {a0})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;

        // The emulator has no system annunciator area to reserve above application content.
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "h", "I", 0).await?;

        Ok(())
    }

    #[method(public)]
    async fn show(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::show({this:?})");

        Ok(())
    }
}
