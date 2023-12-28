use alloc::vec;

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::{JavaObjectProxy, JvmArrayClassInstanceProxy},
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

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Main>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn main(context: &mut dyn JavaContext, this: JavaObjectProxy<Main>, args: JvmArrayClassInstanceProxy<String>) -> JavaResult<()> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Main::<init>({:#x}, {:#x})",
            this.ptr_instance,
            context.instance_raw(&args.class_instance)
        );

        let jlet = context
            .jvm()
            .invoke_static_method("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", &[])
            .await?;
        let event_queue = context
            .jvm()
            .invoke_method(
                &jlet.as_object().unwrap(),
                "org/kwis/msp/lcdui/Jlet",
                "getEventQueue",
                "()Lorg/kwis/msp/lcdui/EventQueue;",
                &[],
            )
            .await?;

        let event = context.jvm().instantiate_array("I", 4).await?;

        loop {
            context
                .jvm()
                .invoke_method(
                    event_queue.as_object_ref().unwrap(),
                    "org/kwis/lcdui/EventQueue",
                    "getNextEvent",
                    "([I)V",
                    &[JavaValue::Object(Some(event.clone()))],
                )
                .await?;
            context
                .jvm()
                .invoke_method(
                    event_queue.as_object_ref().unwrap(),
                    "org/kwis/lcdui/EventQueue",
                    "dispatchEvent",
                    "([I)V",
                    &[JavaValue::Object(Some(event.clone()))],
                )
                .await?;
        }
    }
}
