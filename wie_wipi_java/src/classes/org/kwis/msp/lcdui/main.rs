use alloc::vec;

use java_class_proto::{JavaMethodProto, JavaResult};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm};

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.lcdui.Main
pub struct Main {}

impl Main {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("main", "([Ljava/lang/String;)V", Self::main, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Main>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::<init>({:?})", &this);

        Ok(())
    }

    async fn main(jvm: &Jvm, _: &mut WIPIJavaContext, args: ClassInstanceRef<String>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::main({:?})", &args);

        let jlet = jvm
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;
        let event_queue = jvm
            .invoke_virtual(&jlet, "org/kwis/msp/lcdui/Jlet", "getEventQueue", "()Lorg/kwis/msp/lcdui/EventQueue;", [])
            .await?;

        let event = jvm.instantiate_array("I", 4).await?;

        loop {
            jvm.invoke_virtual(&event_queue, "org/kwis/lcdui/EventQueue", "getNextEvent", "([I)V", [event.clone().into()])
                .await?;
            jvm.invoke_virtual(
                &event_queue,
                "org/kwis/lcdui/EventQueue",
                "dispatchEvent",
                "([I)V",
                [event.clone().into()],
            )
            .await?;
        }
    }
}
