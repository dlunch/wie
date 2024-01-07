use alloc::{boxed::Box, vec};

use java_runtime_base::{
    JavaError, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle, MethodBody,
};
use jvm::{JavaValue, Jvm};

use crate::{classes::org::kwis::msp::lcdui::EventQueue, WieClassProto, WieContext};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
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

    async fn init(jvm: &mut Jvm, _: &mut WieContext, mut this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::<init>({:?})", &this);

        let display = jvm
            .new_class(
                "org/kwis/msp/lcdui/Display",
                "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                (this.clone(), None),
            )
            .await?;

        jvm.put_field(&mut this, "dis", "Lorg/kwis/msp/lcdui/Display;", display)?;

        let event_queue = jvm
            .new_class("org/kwis/msp/lcdui/EventQueue", "(Lorg/kwis/msp/lcdui/Jlet;)V", (this.clone(),))
            .await?;

        jvm.put_field(&mut this, "eq", "Lorg/kwis/msp/lcdui/EventQueue;", event_queue)?;

        jvm.put_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", "Lorg/kwis/msp/lcdui/Jlet;", this.clone())
            .await?;

        Ok(())
    }

    async fn get_active_jlet(jvm: &mut Jvm, _: &mut WieContext) -> JavaResult<JvmClassInstanceHandle<Jlet>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getActiveJlet");

        let jlet = jvm
            .get_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", "Lorg/kwis/msp/lcdui/Jlet;")
            .await?;

        Ok(jlet)
    }

    async fn get_event_queue(
        jvm: &mut Jvm,
        _: &mut WieContext,
        this: JvmClassInstanceHandle<Self>,
    ) -> JavaResult<JvmClassInstanceHandle<EventQueue>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getEventQueue({:?})", &this);

        let eq = jvm.get_field(&this, "eq", "Lorg/kwis/msp/lcdui/EventQueue;")?;

        Ok(eq)
    }

    pub async fn start(jvm: &mut Jvm, context: &mut WieContext, main_class_name: &str) -> JavaResult<()> {
        let main_class_name = main_class_name.replace('.', "/");

        let main_class = jvm.new_class(&main_class_name, "()V", []).await?;

        tracing::debug!("Main class instance: {:?}", &main_class);

        let arg = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        jvm.invoke_virtual(&main_class, &main_class_name, "startApp", "([Ljava/lang/String;)V", [arg.into()])
            .await?;

        struct StartProxy {}

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError, WieContext> for StartProxy {
            #[tracing::instrument(name = "main", skip_all)]
            async fn call(&self, jvm: &mut Jvm, _: &mut WieContext, _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
                jvm.invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", [None.into()])
                    .await?;

                Ok(JavaValue::Void)
            }
        }

        context.spawn(Box::new(StartProxy {}))?;

        Ok(())
    }
}
