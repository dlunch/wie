use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lcdui.Main
pub struct Main;

impl Main {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/Main",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("main", "([Ljava/lang/String;)V", Self::main, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Main>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::<init>({:?})", &this);

        Ok(())
    }

    async fn main(jvm: &Jvm, _: &mut WieJvmContext, args: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::main({:?})", &args);

        let jlet = jvm
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;
        let event_queue = jvm
            .invoke_virtual(&jlet, "getEventQueue", "()Lorg/kwis/msp/lcdui/EventQueue;", [])
            .await?;

        let event = jvm.instantiate_array("I", 4).await?;

        loop {
            let _: () = jvm.invoke_virtual(&event_queue, "getNextEvent", "([I)V", [event.clone().into()]).await?;
            let _: () = jvm.invoke_virtual(&event_queue, "dispatchEvent", "([I)V", [event.clone().into()]).await?;
        }
    }
}
