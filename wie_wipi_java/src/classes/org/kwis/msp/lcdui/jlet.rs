use alloc::{boxed::Box, vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto, MethodBody};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, ClassInstanceRef, JavaError, JavaValue, Jvm, Result as JvmResult};

use crate::{
    classes::org::kwis::msp::lcdui::EventQueue,
    context::{WIPIJavaClassProto, WIPIJavaContext},
};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "getActiveJlet",
                    "()Lorg/kwis/msp/lcdui/Jlet;",
                    Self::get_active_jlet,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getEventQueue",
                    "()Lorg/kwis/msp/lcdui/EventQueue;",
                    Self::get_event_queue,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "getAppProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    Self::get_app_property,
                    Default::default(),
                ),
            ],
            fields: vec![
                JavaFieldProto::new("dis", "Lorg/kwis/msp/lcdui/Display;", Default::default()),
                JavaFieldProto::new("eq", "Lorg/kwis/msp/lcdui/EventQueue;", Default::default()),
                JavaFieldProto::new("qtletActive", "Lorg/kwis/msp/lcdui/Jlet;", FieldAccessFlags::STATIC),
            ],
        }
    }

    async fn init(jvm: &Jvm, context: &mut WIPIJavaContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::<init>({:?})", &this);

        let display = jvm
            .new_class(
                "org/kwis/msp/lcdui/Display",
                "(Lorg/kwis/msp/lcdui/Jlet;Lorg/kwis/msp/lcdui/DisplayProxy;)V",
                (this.clone(), None),
            )
            .await?;

        jvm.put_field(&mut this, "dis", "Lorg/kwis/msp/lcdui/Display;", display).await?;

        let event_queue = jvm
            .new_class("org/kwis/msp/lcdui/EventQueue", "(Lorg/kwis/msp/lcdui/Jlet;)V", (this.clone(),))
            .await?;

        jvm.put_field(&mut this, "eq", "Lorg/kwis/msp/lcdui/EventQueue;", event_queue).await?;

        jvm.put_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", "Lorg/kwis/msp/lcdui/Jlet;", this.clone())
            .await?;

        struct MainProxy {}
        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError, WIPIJavaContext> for MainProxy {
            #[tracing::instrument(name = "main", skip_all)]
            async fn call(&self, jvm: &Jvm, context: &mut WIPIJavaContext, _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
                let now = context.system().platform().now();
                let until = now + 10;
                context.system().sleep(until).await; // XXX wait until jlet to initialize

                jvm.invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", [None.into()])
                    .await?;

                Ok(JavaValue::Void)
            }
        }

        context.spawn(Box::new(MainProxy {}))?;

        Ok(())
    }

    async fn get_active_jlet(jvm: &Jvm, _: &mut WIPIJavaContext) -> JvmResult<ClassInstanceRef<Jlet>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getActiveJlet");

        let jlet = jvm
            .get_static_field("org/kwis/msp/lcdui/Jlet", "qtletActive", "Lorg/kwis/msp/lcdui/Jlet;")
            .await?;

        Ok(jlet)
    }

    async fn get_event_queue(jvm: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<EventQueue>> {
        tracing::debug!("org.kwis.msp.lcdui.Jlet::getEventQueue({:?})", &this);

        let eq = jvm.get_field(&this, "eq", "Lorg/kwis/msp/lcdui/EventQueue;").await?;

        Ok(eq)
    }

    async fn get_app_property(
        jvm: &Jvm,
        _: &mut WIPIJavaContext,
        this: ClassInstanceRef<Self>,
        key: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub org.kwis.msp.lcdui.Jlet::getAppProperty({:?}, {:?})", &this, &key);

        Ok(JavaLangString::from_rust_string(jvm, "").await?.into())
    }
}
