use alloc::{boxed::Box, format, vec};

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaContext, JavaError, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    method::MethodBody,
    proxy::JvmClassInstanceProxy,
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

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::<init>({:#x})", context.instance_raw(&this.class_instance));

        let display = context.instantiate("Lorg/kwis/msp/lcdui/Display;").await?;
        context
            .call_method(
                &display.cast(),
                "<init>",
                "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                &[context.instance_raw(&this.class_instance), 0],
            )
            .await?;
        let display = context.instance_from_raw(display.ptr_instance);

        context.jvm().put_field(
            &this.class_instance,
            "dis",
            "Lorg/kwis/msp/lcdui/Display;",
            JavaValue::Object(Some(display)),
        )?;

        let event_queue = context.instantiate("Lorg/kwis/msp/lcdui/EventQueue;").await?;
        context
            .call_method(
                &event_queue.cast(),
                "<init>",
                "(Lorg/kwis/msp/lcdui/Jlet;)V",
                &[context.instance_raw(&this.class_instance)],
            )
            .await?;

        let event_queue = context.instance_from_raw(event_queue.ptr_instance);
        context.jvm().put_field(
            &this.class_instance,
            "eq",
            "Lorg/kwis/msp/lcdui/EventQueue;",
            JavaValue::Object(Some(event_queue)),
        )?;

        context
            .jvm()
            .put_static_field(
                "org/kwis/msp/lcdui/Jlet",
                "qtletActive",
                "Lorg/kwis/msp/lcdui/Jlet;",
                JavaValue::Object(Some(this.class_instance)),
            )
            .await?;

        Ok(())
    }

    async fn get_active_jlet(context: &mut dyn JavaContext) -> JavaResult<JavaObjectProxy<Jlet>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getActiveJlet");

        let jlet = context
            .jvm()
            .get_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", "Lorg/kwis/msp/lcdui/Jlet;")
            .await?;
        let instance = context.instance_raw(jlet.as_object_ref().unwrap());

        Ok(JavaObjectProxy::new(instance))
    }

    async fn get_event_queue(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<JvmClassInstanceProxy<EventQueue>> {
        tracing::debug!(
            "org.kwis.msp.lcdui.Jlet::getEventQueue({:#x})",
            context.instance_raw(&this.class_instance)
        );

        let eq = context.jvm().get_field(&this.class_instance, "eq", "Lorg/kwis/msp/lcdui/EventQueue;")?;

        Ok(JvmClassInstanceProxy::new(eq.as_object_ref().unwrap().clone()))
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
