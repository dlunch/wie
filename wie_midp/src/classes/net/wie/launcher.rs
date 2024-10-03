use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class net.wie.Launcher
pub struct Launcher;

impl Launcher {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/Launcher",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("start", "(Ljava/lang/String;)V", Self::start, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "startMIDlet",
                    "(Ljavax/microedition/midlet/MIDlet;)V",
                    Self::start_midlet,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
        }
    }

    async fn start(jvm: &Jvm, _context: &mut WieJvmContext, main_class: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.Launcher::start({:?})", &main_class);

        // create main class
        let main_class = JavaLangString::to_rust_string(jvm, &main_class).await?;
        let main_class = jvm.new_class(&main_class, "()V", ()).await?;

        jvm.invoke_static("net/wie/Launcher", "startMIDlet", "(Ljavax/microedition/midlet/MIDlet;)V", (main_class,))
            .await
    }

    async fn start_midlet(jvm: &Jvm, _context: &mut WieJvmContext, midlet: ClassInstanceRef<()>) -> JvmResult<()> {
        tracing::debug!("net.wie.Launcher::startMIDlet({:?})", &midlet);

        // run startApp
        let _: () = jvm.invoke_virtual(&midlet, "startApp", "()V", (None,)).await?;

        // event loop
        let event_queue = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;

        let event = jvm.instantiate_array("I", 4).await?;
        loop {
            let _: () = jvm.invoke_virtual(&event_queue, "getNextEvent", "([I)V", (event.clone(),)).await?;
            let _: () = jvm.invoke_virtual(&event_queue, "dispatchEvent", "([I)V", (event.clone(),)).await?;
        }
    }
}
