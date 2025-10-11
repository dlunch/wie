use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::Jlet;

// class net.wie.WIPIMIDlet
pub struct WIPIMIDlet;

impl WIPIMIDlet {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WIPIMIDlet",
            parent_class: Some("javax/microedition/midlet/MIDlet"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("startApp", "()V", Self::start_app, Default::default()),
                JavaMethodProto::new("pauseApp", "()V", Self::pause_app, Default::default()),
                JavaMethodProto::new("destroyApp", "(Z)V", Self::destroy_app, Default::default()),
                JavaMethodProto::new(
                    "setCurrentJlet",
                    "(Lorg/kwis/msp/lcdui/Jlet;)V",
                    Self::set_current_jlet,
                    Default::default(),
                ),
            ],
            fields: vec![JavaFieldProto::new("jlet", "Lorg/kwis/msp/lcdui/Jlet;", Default::default())],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::<init>({:?})", this);

        let _: () = jvm.invoke_special(&this, "javax/microedition/midlet/MIDlet", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn set_current_jlet(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        jlet: ClassInstanceRef<Jlet>,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::setCurrentJlet({:?}, {:?})", this, jlet);

        jvm.put_field(&mut this, "jlet", "Lorg/kwis/msp/lcdui/Jlet;", jlet).await?;

        Ok(())
    }

    async fn start_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::startApp({:?})", this);

        let args_array = jvm.instantiate_array("Ljava/lang/String;", 0).await?;

        let jlet = jvm.get_field(&this, "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        let _: () = jvm.invoke_virtual(&jlet, "startApp", "([Ljava/lang/String;)V", (args_array,)).await?;

        Ok(())
    }

    async fn pause_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::pauseApp({:?})", this);

        let jlet = jvm.get_field(&this, "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        let _: () = jvm.invoke_virtual(&jlet, "pauseApp", "()V", ()).await?;

        Ok(())
    }

    async fn destroy_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, unconditional: bool) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIMIDlet::destroyApp({:?}, {:?})", this, unconditional);

        let jlet = jvm.get_field(&this, "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        let _: () = jvm.invoke_virtual(&jlet, "destroyApp", "(Z)V", (unconditional,)).await?;

        Ok(())
    }
}
