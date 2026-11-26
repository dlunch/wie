use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::org::kwis::msp::lcdui::Jlet;

// class net.wie.WIPIMIDlet
#[java_class(name = "net/wie/WIPIMIDlet", extends = "javax/microedition/midlet/MIDlet", public)]
pub struct WIPIMIDlet {
    #[field(private)]
    jlet: ClassInstanceRef<Jlet>,
    #[field(private)]
    destroyed: bool,
}

#[java_methods]
impl WIPIMIDlet {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/midlet/MIDlet", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[method(public)]
    async fn set_current_jlet(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        jlet: ClassInstanceRef<Jlet>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::setCurrentJlet({this:?}, {jlet:?})");

        jvm.put_field(&mut this, "net/wie/WIPIMIDlet", "jlet", "Lorg/kwis/msp/lcdui/Jlet;", jlet)
            .await?;

        Ok(())
    }

    #[method(protected)]
    async fn start_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::startApp({this:?})");

        let args_array = jvm.instantiate_array("Ljava/lang/String;", 0).await?;

        let jlet: ClassInstanceRef<Jlet> = jvm.get_field(&this, "net/wie/WIPIMIDlet", "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        let _: () = jvm
            .invoke_virtual(&jlet, "org/kwis/msp/lcdui/Jlet", "startApp", "([Ljava/lang/String;)V", (args_array,))
            .await?;

        Ok(())
    }

    #[method(protected)]
    async fn pause_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::pauseApp({this:?})");

        let jlet: ClassInstanceRef<Jlet> = jvm.get_field(&this, "net/wie/WIPIMIDlet", "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        let _: () = jvm.invoke_virtual(&jlet, "org/kwis/msp/lcdui/Jlet", "pauseApp", "()V", ()).await?;

        Ok(())
    }

    #[method(protected)]
    async fn destroy_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, unconditional: bool) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::destroyApp({this:?}, {unconditional:?})");

        let jlet = jvm.get_field(&this, "net/wie/WIPIMIDlet", "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        Jlet::destroy(jvm, this, jlet, unconditional).await
    }
}
