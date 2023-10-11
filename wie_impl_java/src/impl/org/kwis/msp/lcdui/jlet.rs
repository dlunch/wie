use alloc::{boxed::Box, format, vec};

use crate::{
    base::{JavaClassProto, JavaContext, JavaError, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    method::MethodBody,
    r#impl::org::kwis::msp::lcdui::EventQueue,
    JavaFieldAccessFlag, JavaObjectProxy,
};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "getActiveJlet",
                    "()Lorg/kwis/msp/lcdui/Jlet;",
                    Self::get_active_jlet,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new(
                    "getEventQueue",
                    "()Lorg/kwis/msp/lcdui/EventQueue;",
                    Self::get_event_queue,
                    JavaMethodFlag::NONE,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("dis", "Lorg/kwis/msp/lcdui/Display;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("eq", "Lorg/kwis/msp/lcdui/EventQueue;", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("qtletActive", "Lorg/kwis/msp/lcdui/Jlet;", JavaFieldAccessFlag::STATIC),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<Jlet>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::<init>");

        let display = context.instantiate("Lorg/kwis/msp/lcdui/Display;").await?;
        context
            .call_method(
                &display.cast(),
                "<init>",
                "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                &[this.ptr_instance, 0],
            )
            .await?;

        let field_id = context.get_field_id("org/kwis/msp/lcdui/Jlet", "dis", "Lorg/kwis/msp/lcdui/Display;")?;
        context.put_field_by_id(&this.cast(), field_id, display.ptr_instance)?;

        let event_queue = context.instantiate("Lorg/kwis/msp/lcdui/EventQueue;").await?;
        context
            .call_method(&event_queue.cast(), "<init>", "(Lorg/kwis/msp/lcdui/Jlet;)V", &[this.ptr_instance])
            .await?;

        let field_id = context.get_field_id("org/kwis/msp/lcdui/Jlet", "eq", "Lorg/kwis/msp/lcdui/EventQueue;")?;
        context.put_field_by_id(&this.cast(), field_id, event_queue.ptr_instance)?;

        context.put_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", this.ptr_instance)?;

        Ok(())
    }

    async fn get_active_jlet(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Jlet>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getActiveJlet");

        let jlet = JavaObjectProxy::new(context.get_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive")?);
        Ok(jlet)
    }

    async fn get_event_queue(context: &mut dyn JavaContext, this: JavaObjectProxy<Jlet>) -> JavaResult<JavaObjectProxy<EventQueue>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getEventQueue");

        let field_id = context.get_field_id("org/kwis/msp/lcdui/Jlet", "eq", "Lorg/kwis/msp/lcdui/EventQueue;")?;
        let eq = JavaObjectProxy::new(context.get_field_by_id(&this.cast(), field_id)?);

        Ok(eq)
    }

    pub async fn start(context: &mut dyn JavaContext, main_class_name: &str) -> JavaResult<()> {
        let main_class_name = main_class_name.replace('.', "/");
        let ptr_main_class = context.instantiate(&format!("L{};", main_class_name)).await?;
        context.call_method(&ptr_main_class, "<init>", "()V", &[]).await?;

        tracing::debug!("Main class instance: {:#x}", ptr_main_class.ptr_instance);

        let arg = context.instantiate_array("Ljava/lang/String;", 0).await?;
        context
            .call_method(&ptr_main_class, "startApp", "([Ljava/lang/String;)V", &[arg.ptr_instance])
            .await?;

        struct StartProxy {}

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError> for StartProxy {
            #[tracing::instrument(name = "main", skip_all)]
            async fn call(&self, context: &mut dyn JavaContext, _: &[usize]) -> Result<usize, JavaError> {
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
