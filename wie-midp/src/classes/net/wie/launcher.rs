use alloc::boxed::Box;

use jvm::{ClassInstanceRef, JavaError, JavaValue, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{MethodBody, java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::{javax::microedition::midlet::MIDlet, net::wie::EventQueue};

// class net.wie.Launcher
#[java_class(name = "net/wie/Launcher", public)]
pub struct Launcher;

#[java_methods]
impl Launcher {
    #[method(public, static)]
    async fn start(jvm: &Jvm, _context: &mut WieJvmContext, main_class: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.Launcher::start({main_class:?})");

        // create main class
        let main_class = JavaLangString::to_rust_string(jvm, &main_class).await?;
        let main_class = jvm.new_class(&main_class, "()V", ()).await?;

        jvm.invoke_static("net/wie/Launcher", "startMIDlet", "(Ljavax/microedition/midlet/MIDlet;)V", (main_class,))
            .await
    }

    #[method(name = "startMIDlet", public, static)]
    async fn start_midlet(jvm: &Jvm, context: &mut WieJvmContext, midlet: ClassInstanceRef<MIDlet>) -> JvmResult<()> {
        tracing::debug!("net.wie.Launcher::startMIDlet({midlet:?})");

        // run startApp
        let _: () = jvm
            .invoke_virtual(&midlet, "javax/microedition/midlet/MIDlet", "startApp", "()V", ())
            .await?;

        // spawn event loop
        context.spawn(jvm, Box::new(EventLoopRunner))?;

        Ok(())
    }
}

struct EventLoopRunner;

#[async_trait::async_trait]
impl MethodBody<JavaError, WieJvmContext> for EventLoopRunner {
    async fn call(&self, jvm: &Jvm, _context: &mut WieJvmContext, _args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        jvm.attach_thread(None).await?;

        // event loop
        let event_queue: ClassInstanceRef<EventQueue> = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;

        let event = jvm.instantiate_array("I", 4).await?;
        loop {
            let _: () = jvm
                .invoke_virtual(&event_queue, "net/wie/EventQueue", "getNextEvent", "([I)V", (event.clone(),))
                .await?;
            let _: () = jvm
                .invoke_virtual(&event_queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event.clone(),))
                .await?;
        }
    }
}
