use alloc::vec;

use jvm::{ClassInstanceRef, JavaValue};

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JvmClassInstanceProxy,
    r#impl::java::lang::String,
};

// class org.kwis.msp.lcdui.Main
pub struct Main {}

impl Main {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("main", "([Ljava/lang/String;)V", Self::main, JavaMethodFlag::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Main>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::<init>({:?})", &this);

        Ok(())
    }

    async fn main(context: &mut dyn JavaContext, args: JvmClassInstanceProxy<String>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::main({:?})", &args);

        let jlet: ClassInstanceRef = context
            .jvm()
            .invoke_static("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", [])
            .await?;
        let event_queue: ClassInstanceRef = context
            .jvm()
            .invoke_virtual(&jlet, "org/kwis/msp/lcdui/Jlet", "getEventQueue", "()Lorg/kwis/msp/lcdui/EventQueue;", [])
            .await?;

        let event = context.jvm().instantiate_array("I", 4).await?;

        loop {
            context
                .jvm()
                .invoke_virtual(
                    &event_queue,
                    "org/kwis/lcdui/EventQueue",
                    "getNextEvent",
                    "([I)V",
                    [JavaValue::Object(Some(event.clone()))],
                )
                .await?;
            context
                .jvm()
                .invoke_virtual(
                    &event_queue,
                    "org/kwis/lcdui/EventQueue",
                    "dispatchEvent",
                    "([I)V",
                    [JavaValue::Object(Some(event.clone()))],
                )
                .await?;
        }
    }
}
