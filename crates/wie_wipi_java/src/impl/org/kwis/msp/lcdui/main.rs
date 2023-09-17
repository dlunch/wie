use alloc::{boxed::Box, vec};

use crate::{
    base::{JavaClassProto, JavaContext, JavaError, JavaMethodFlag, JavaMethodProto, JavaResult},
    method::MethodBody,
    proxy::JavaObjectProxy,
    Array,
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

    async fn main(context: &mut dyn JavaContext, this: JavaObjectProxy<Main>, args: JavaObjectProxy<Array>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Main::<init>({:#x}, {:#x})", this.ptr_instance, args.ptr_instance);

        let jlet = JavaObjectProxy::new(
            context
                .call_static_method("org/kwis/msp/lcdui/Jlet", "getActiveJlet", "()Lorg/kwis/msp/lcdui/Jlet;", &[])
                .await?,
        );
        let event_queue = JavaObjectProxy::new(
            context
                .call_method(&jlet, "getEventQueue", "()Lorg/kwis/msp/lcdui/EventQueue;", &[])
                .await?,
        );

        let event = context.instantiate_array("I", 4).await?;

        loop {
            context.call_method(&event_queue, "getNextEvent", "([I)V", &[event.ptr_instance]).await?;
            context.call_method(&event_queue, "dispatchEvent", "([I)V", &[event.ptr_instance]).await?;
        }
    }

    pub async fn start(context: &mut dyn JavaContext) -> JavaResult<()> {
        struct StartProxy {}

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError> for StartProxy {
            #[tracing::instrument(name = "main", skip_all)]
            async fn call(&self, context: &mut dyn JavaContext, _: &[u32]) -> Result<u32, JavaError> {
                context
                    .call_static_method("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", &[])
                    .await?;

                Ok(0)
            }
        }

        context.spawn(Box::new(StartProxy {}))?;

        Ok(())
    }
}
